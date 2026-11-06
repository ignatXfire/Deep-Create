//! Real Win32 Named Pipe server and client.
//!
//! This module performs the actual OS calls that the C-12 identity boundary
//! requires to *acquire* a peer principal at the channel layer:
//! `CreateNamedPipeW`, `ConnectNamedPipe`, `CreateFileW`, `ReadFile`,
//! `WriteFile`, `GetNamedPipeClientProcessId` and `ImpersonateNamedPipeClient`.
//!
//! The pure typed peer-identity comparison lives in `identity.rs`. This
//! module is the OS-side acquisition adapter that produces a verified
//! channel endpoint; the caller still has to feed the peer identity into
//! `identity::authenticate_peer` for the typed check.
//!
//! Safety model: `unsafe_code` is denied workspace-wide. The ONLY unsafe
//! blocks live in this file, each explicitly allowed and justified. Every
//! handle is wrapped in RAII so it is closed exactly once.
//!
//! Compilation: Windows-only (`#[cfg(windows)]`). On other platforms this
//! module is empty.

#![cfg(windows)]
// SAFETY-POLICY: This module is an audited unsafe boundary of the workspace.
// Each Win32 FFI call requires `unsafe`; the workspace-level
// `unsafe_code = "deny"` is overridden ONLY here, block by block, with an
// accompanying safety justification at every call site.
#![allow(unsafe_code)]

use windows_sys::Win32::Foundation::{
    CloseHandle, GetLastError, HANDLE, INVALID_HANDLE_VALUE, ERROR_PIPE_CONNECTED,
};
use windows_sys::Win32::Storage::FileSystem::{
    CreateFileW, FlushFileBuffers, ReadFile, WriteFile,
    FILE_GENERIC_READ, FILE_GENERIC_WRITE, OPEN_EXISTING, PIPE_ACCESS_DUPLEX,
};
use windows_sys::Win32::Security::RevertToSelf;
use windows_sys::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, DisconnectNamedPipe, GetNamedPipeClientProcessId,
    ImpersonateNamedPipeClient, PIPE_READMODE_MESSAGE, PIPE_TYPE_MESSAGE,
    PIPE_UNLIMITED_INSTANCES, PIPE_WAIT,
};

const PIPE_BUFFER_SIZE: u32 = 64 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum PipeError {
    #[error("named pipe creation failed (os error {0})")]
    CreationFailed(u32),
    #[error("named pipe accept failed (os error {0})")]
    AcceptFailed(u32),
    #[error("named pipe connect failed (os error {0})")]
    ConnectFailed(u32),
    #[error("named pipe I/O failed (os error {0})")]
    IoFailed(u32),
    #[error("peer process id query failed (os error {0})")]
    PeerPidQueryFailed(u32),
    #[error("peer impersonation failed (os error {0})")]
    ImpersonationFailed(u32),
}

struct OwnedHandle(HANDLE);

impl OwnedHandle {
    fn new(h: HANDLE, err: PipeError) -> Result<Self, PipeError> {
        if h.is_null() || h == INVALID_HANDLE_VALUE {
            return Err(err);
        }
        Ok(Self(h))
    }
    fn raw(&self) -> HANDLE {
        self.0
    }
}

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        if !self.0.is_null() && self.0 != INVALID_HANDLE_VALUE {
            // SAFETY: we own the handle and close it exactly once here.
            unsafe { CloseHandle(self.0) };
        }
    }
}

fn os_err() -> u32 {
    // SAFETY: GetLastError is thread-local and safe to call.
    unsafe { GetLastError() }
}

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub struct NamedPipeServer {
    handle: OwnedHandle,
}

pub struct PipeConnection {
    handle: OwnedHandle,
}

pub struct NamedPipeClient {
    handle: OwnedHandle,
}

impl NamedPipeServer {
    pub fn create(path: &str) -> Result<Self, PipeError> {
        let wide = to_wide(path);
        // SAFETY: `wide` is a valid null-terminated UTF-16 buffer that
        // outlives the call. No security attributes (default DACL).
        let h = unsafe {
            CreateNamedPipeW(
                wide.as_ptr(),
                PIPE_ACCESS_DUPLEX,
                PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_WAIT,
                PIPE_UNLIMITED_INSTANCES,
                PIPE_BUFFER_SIZE,
                PIPE_BUFFER_SIZE,
                0,
                std::ptr::null(),
            )
        };
        let last = os_err();
        Ok(Self {
            handle: OwnedHandle::new(h, PipeError::CreationFailed(last))?,
        })
    }

    pub fn accept(self) -> Result<PipeConnection, PipeError> {
        // SAFETY: valid server handle. ConnectNamedPipe blocks until a
        // client connects, or returns 0 with ERROR_PIPE_CONNECTED if a
        // client raced in between CreateNamedPipeW and this call.
        let ok = unsafe { ConnectNamedPipe(self.handle.raw(), std::ptr::null_mut()) };
        if ok == 0 {
            let e = os_err();
            if e != ERROR_PIPE_CONNECTED {
                return Err(PipeError::AcceptFailed(e));
            }
        }
        Ok(PipeConnection {
            handle: self.handle,
        })
    }
}

impl PipeConnection {
    pub fn read(&self, buf: &mut [u8]) -> Result<usize, PipeError> {
        let mut n: u32 = 0;
        // SAFETY: valid pipe handle, `buf` is a valid writable slice.
        let ok = unsafe {
            ReadFile(
                self.handle.raw(),
                buf.as_mut_ptr(),
                buf.len() as u32,
                &mut n,
                std::ptr::null_mut(),
            )
        };
        if ok == 0 {
            return Err(PipeError::IoFailed(os_err()));
        }
        Ok(n as usize)
    }

    pub fn write(&self, buf: &[u8]) -> Result<usize, PipeError> {
        let mut n: u32 = 0;
        // SAFETY: valid pipe handle, `buf` is a valid readable slice.
        let ok = unsafe {
            WriteFile(
                self.handle.raw(),
                buf.as_ptr(),
                buf.len() as u32,
                &mut n,
                std::ptr::null_mut(),
            )
        };
        if ok == 0 {
            return Err(PipeError::IoFailed(os_err()));
        }
        // SAFETY: valid pipe handle.
        unsafe { FlushFileBuffers(self.handle.raw()) };
        Ok(n as usize)
    }

    pub fn client_process_id(&self) -> Result<u32, PipeError> {
        let mut pid: u32 = 0;
        // SAFETY: valid server-side pipe handle, `pid` is a valid out-ptr.
        let ok = unsafe { GetNamedPipeClientProcessId(self.handle.raw(), &mut pid) };
        if ok == 0 {
            return Err(PipeError::PeerPidQueryFailed(os_err()));
        }
        Ok(pid)
    }

    pub fn impersonate(&self) -> Result<ImpersonationGuard, PipeError> {
        // SAFETY: valid server-side pipe handle. Must be paired with
        // RevertToSelf on the same thread, which ImpersonationGuard::drop
        // guarantees.
        let ok = unsafe { ImpersonateNamedPipeClient(self.handle.raw()) };
        if ok == 0 {
            return Err(PipeError::ImpersonationFailed(os_err()));
        }
        Ok(ImpersonationGuard { _private: () })
    }
}

impl Drop for PipeConnection {
    fn drop(&mut self) {
        // SAFETY: valid pipe handle; failure is silent (close proceeds).
        unsafe { DisconnectNamedPipe(self.handle.raw()) };
    }
}

pub struct ImpersonationGuard {
    _private: (),
}

impl Drop for ImpersonationGuard {
    fn drop(&mut self) {
        // SAFETY: pairs with the ImpersonateNamedPipeClient call that
        // produced this guard, on the same thread.
        unsafe { RevertToSelf() };
    }
}

impl NamedPipeClient {
    pub fn connect(path: &str) -> Result<Self, PipeError> {
        let wide = to_wide(path);
        // SAFETY: `wide` is a valid null-terminated UTF-16 buffer.
        let h = unsafe {
            CreateFileW(
                wide.as_ptr(),
                FILE_GENERIC_READ | FILE_GENERIC_WRITE,
                0,
                std::ptr::null(),
                OPEN_EXISTING,
                0,
                std::ptr::null_mut(),
            )
        };
        let last = os_err();
        Ok(Self {
            handle: OwnedHandle::new(h, PipeError::ConnectFailed(last))?,
        })
    }

    pub fn read(&self, buf: &mut [u8]) -> Result<usize, PipeError> {
        let mut n: u32 = 0;
        // SAFETY: valid pipe handle, `buf` is a valid writable slice.
        let ok = unsafe {
            ReadFile(
                self.handle.raw(),
                buf.as_mut_ptr(),
                buf.len() as u32,
                &mut n,
                std::ptr::null_mut(),
            )
        };
        if ok == 0 {
            return Err(PipeError::IoFailed(os_err()));
        }
        Ok(n as usize)
    }

    pub fn write(&self, buf: &[u8]) -> Result<usize, PipeError> {
        let mut n: u32 = 0;
        // SAFETY: valid pipe handle, `buf` is a valid readable slice.
        let ok = unsafe {
            WriteFile(
                self.handle.raw(),
                buf.as_ptr(),
                buf.len() as u32,
                &mut n,
                std::ptr::null_mut(),
            )
        };
        if ok == 0 {
            return Err(PipeError::IoFailed(os_err()));
        }
        Ok(n as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::thread;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    fn fresh_pipe_path(tag: &str) -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!(r"\\.\pipe\dc-windows-test-{}-{}", tag, nanos)
    }

    #[test]
    fn server_client_round_trip() {
        let path = fresh_pipe_path("roundtrip");
        let (tx_path, rx_path) = mpsc::channel::<()>();
        let (tx_done, rx_done) = mpsc::channel::<()>();

        let server_path = path.clone();
        let server = thread::spawn(move || {
            let srv = NamedPipeServer::create(&server_path).expect("server create");
            tx_path.send(()).unwrap();
            let conn = srv.accept().expect("accept");
            let mut buf = [0u8; 64];
            let n = conn.read(&mut buf).expect("server read");
            assert_eq!(&buf[..n], b"ping");
            let written = conn.write(b"pong").expect("server write");
            assert_eq!(written, 4);
            tx_done.send(()).unwrap();
        });

        rx_path.recv().unwrap();
        thread::sleep(Duration::from_millis(50));

        let client = NamedPipeClient::connect(&path).expect("client connect");
        client.write(b"ping").expect("client write");
        let mut buf = [0u8; 64];
        let n = client.read(&mut buf).expect("client read");
        assert_eq!(&buf[..n], b"pong");

        rx_done.recv().unwrap();
        server.join().unwrap();
    }

    #[test]
    fn server_reports_real_client_pid() {
        let path = fresh_pipe_path("pid");
        let (tx_path, rx_path) = mpsc::channel::<()>();
        let (tx_pid, rx_pid) = mpsc::channel::<u32>();

        let server_path = path.clone();
        let server = thread::spawn(move || {
            let srv = NamedPipeServer::create(&server_path).expect("server create");
            tx_path.send(()).unwrap();
            let conn = srv.accept().expect("accept");
            let pid = conn.client_process_id().expect("client pid");
            tx_pid.send(pid).unwrap();
            thread::sleep(Duration::from_millis(50));
        });

        rx_path.recv().unwrap();
        thread::sleep(Duration::from_millis(50));
        let _client = NamedPipeClient::connect(&path).expect("client connect");

        let reported = rx_pid.recv().unwrap();
        assert_eq!(reported, std::process::id(), "client pid must be this process");
        server.join().unwrap();
    }
}
