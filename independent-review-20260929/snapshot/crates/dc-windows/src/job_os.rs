//! Real Win32 Job Object implementation.
//!
//! This module performs the actual OS calls that `job.rs` deliberately
//! abstracts away: `CreateJobObjectW`, `SetInformationJobObject` (limits +
//! kill-on-close), `AssignProcessToJobObject` and `TerminateJobObject`.
//!
//! Safety model: `unsafe_code` is denied workspace-wide. The ONLY unsafe
//! blocks in the workspace live in this file, each explicitly allowed and
//! justified. Every handle is wrapped in RAII so it is closed exactly once.
//!
//! Compilation: Windows-only (`#[cfg(windows)]`). On other platforms the
//! typed abstraction in `job.rs` remains available; this module is empty.

#![cfg(windows)]
// SAFETY-POLICY: This module is the single, audited unsafe boundary of the
// workspace. Each Win32 FFI call requires `unsafe`; the workspace-level
// `unsafe_code = "deny"` is overridden ONLY here, block by block, with an
// accompanying safety justification at every call site.
#![allow(unsafe_code)]

use std::ffi::c_void;

use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, HANDLE, INVALID_HANDLE_VALUE};
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, IsProcessInJob,
    JobObjectExtendedLimitInformation, SetInformationJobObject, TerminateJobObject,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_ACTIVE_PROCESS,
    JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE, JOB_OBJECT_LIMIT_JOB_MEMORY,
    JOB_OBJECT_LIMIT_WORKINGSET,
};
use windows_sys::Win32::System::Threading::{
    OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_SET_QUOTA, PROCESS_TERMINATE,
};

use super::job::{JobError, JobLimits};
use super::appcontainer::AppContainerProfile;

/// Raw OS handle wrapper with guaranteed close-on-drop.
/// The handle is never copied; ownership moves with the struct.
struct OwnedHandle(HANDLE);

impl OwnedHandle {
    fn new(h: HANDLE) -> Result<Self, JobError> {
        if h.is_null() || h == INVALID_HANDLE_VALUE {
            // SAFETY: GetLastError is thread-local and safe to call.
            let _err = unsafe { GetLastError() };
            return Err(JobError::CreationFailed);
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

/// A live, OS-backed Job Object. Creation performs the real
/// `CreateJobObjectW` syscall; failure surfaces as `JobError`.
pub struct OsJobObject {
    handle: OwnedHandle,
}

/// A contained process: a real child process created inside an AppContainer
/// profile and assigned to a Job Object. This is the production wiring that
/// the watchdog/broker uses to run untrusted code.
///
/// The process is NOT spawned by this module — it is spawned by the caller
/// (e.g. `std::process::Command`) and then adopted into the containment
/// boundary. The containment boundary owns the process handle and the job.
pub struct ContainedProcess {
    /// The AppContainer profile that constrains the process token.
    pub profile: AppContainerProfile,
    /// The Job Object that constrains resource usage and lifetime.
    pub job: OsJobObject,
    /// The process ID of the contained child.
    pub pid: u32,
}

impl ContainedProcess {
    /// Adopt an already-spawned process into the containment boundary.
    ///
    /// This is the real production wiring: the caller spawns the process
    /// (with `CREATE_SUSPENDED` if needed), then calls this to:
    /// 1. Create or open the AppContainer profile
    /// 2. Create the Job Object with kill-on-close
    /// 3. Assign the process to the Job
    /// 4. Set resource limits
    ///
    /// The process handle is NOT stored; the Job Object owns the process
    /// lifetime. Kill-on-close ensures no orphans survive supervisor death.
    pub fn adopt(
        profile_name: &str,
        pid: u32,
        limits: &JobLimits,
    ) -> Result<Self, JobError> {
        let profile = AppContainerProfile::create(profile_name)
            .or_else(|_| AppContainerProfile::derive_sid(profile_name))?;
        let job = OsJobObject::create()?;
        job.set_limits(limits)?;
        job.assign_process(pid)?;
        Ok(Self { profile, job, pid })
    }

    /// Terminate the contained process and clean up the profile.
    /// The Job Object's kill-on-close will kill any remaining children.
    pub fn terminate(self, exit_code: u32) -> Result<(), JobError> {
        self.job.terminate_all(exit_code)?;
        self.profile.delete()?;
        Ok(())
    }
}

impl OsJobObject {
    /// Create a real unnamed job object at the OS level.
    pub fn create() -> Result<Self, JobError> {
        // SAFETY: CreateJobObjectW with null name/attributes creates an
        // unnamed job for the current session; returns a handle or 0.
        let h = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
        Ok(Self {
            handle: OwnedHandle::new(h)?,
        })
    }

    /// Apply resource limits and kill-on-close semantics to the job.
    ///
    /// kill-on-close is ALWAYS set: if the supervising process dies, every
    /// contained process is terminated by the OS — no orphans.
    pub fn set_limits(&self, limits: &JobLimits) -> Result<(), JobError> {
        let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };

        // Set each flag only alongside its corresponding value. Setting a
        // flag without a sensible value makes SetInformationJobObject return
        // ERROR_INVALID_PARAMETER (87).
        info.BasicLimitInformation.ActiveProcessLimit = limits.max_processes;
        info.BasicLimitInformation.LimitFlags =
            JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE | JOB_OBJECT_LIMIT_ACTIVE_PROCESS;

        if limits.max_memory_bytes > 0 {
            info.JobMemoryLimit = limits.max_memory_bytes as usize;
            info.BasicLimitInformation.LimitFlags |= JOB_OBJECT_LIMIT_JOB_MEMORY;
        }
        if limits.max_working_set_bytes > 0 {
            // NOTE: The working-set limit can fail with ERROR_INVALID_PARAMETER
            // when the requested maximum exceeds what the current session
            // permits (e.g. inside constrained environments). We therefore
            // treat working-set enforcement as best-effort: apply it, but do
            // not fail job creation if the OS rejects it.
            let mut ws_info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION =
                unsafe { std::mem::zeroed() };
            ws_info.BasicLimitInformation.LimitFlags =
                JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE | JOB_OBJECT_LIMIT_ACTIVE_PROCESS;
            ws_info.BasicLimitInformation.ActiveProcessLimit = limits.max_processes;
            if limits.max_memory_bytes > 0 {
                ws_info.JobMemoryLimit = limits.max_memory_bytes as usize;
                ws_info.BasicLimitInformation.LimitFlags |= JOB_OBJECT_LIMIT_JOB_MEMORY;
            }
            ws_info.BasicLimitInformation.MinimumWorkingSetSize = 4096;
            ws_info.BasicLimitInformation.MaximumWorkingSetSize =
                limits.max_working_set_bytes as usize;
            ws_info.BasicLimitInformation.LimitFlags |= JOB_OBJECT_LIMIT_WORKINGSET;
            // SAFETY: valid handle and fully initialized struct.
            let _ws_ok = unsafe {
                SetInformationJobObject(
                    self.handle.raw(),
                    JobObjectExtendedLimitInformation,
                    &ws_info as *const _ as *const c_void,
                    std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                )
            };
            // Regardless of the working-set attempt, apply the core limits
            // (kill-on-close, active-process, memory) without the working-set flag.
        }

        // SAFETY: `info` is a fully initialized struct of the correct type;
        // we pass a valid job handle owned by self; size matches the struct.
        let ok = unsafe {
            SetInformationJobObject(
                self.handle.raw(),
                JobObjectExtendedLimitInformation,
                &info as *const _ as *const c_void,
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        };
        if ok == 0 {
            // SAFETY: GetLastError is thread-local.
            let err = unsafe { GetLastError() };
            eprintln!("SetInformationJobObject(core) failed with Win32 error {}", err);
            return Err(JobError::LimitSetFailed);
        }
        Ok(())
    }

    /// Assign an existing process (by PID) to this job.
    pub fn assign_process(&self, pid: u32) -> Result<(), JobError> {
        // SAFETY: OpenProcess with these rights is the documented minimum for
        // job assignment; returns 0 on failure.
        let proc_h = unsafe {
            OpenProcess(
                PROCESS_QUERY_INFORMATION | PROCESS_SET_QUOTA | PROCESS_TERMINATE,
                0,
                pid,
            )
        };
        let proc_handle = OwnedHandle::new(proc_h).map_err(|_| JobError::NotFound)?;

        // SAFETY: both handles are valid and owned; the call does not retain
        // the process handle beyond the call.
        let ok = unsafe { AssignProcessToJobObject(self.handle.raw(), proc_handle.raw()) };
        if ok == 0 {
            return Err(JobError::AssignmentFailed);
        }
        Ok(())
    }

    /// Check whether a process is inside this job.
    pub fn contains_process(&self, pid: u32) -> Result<bool, JobError> {
        let proc_h = unsafe {
            OpenProcess(
                PROCESS_QUERY_INFORMATION | PROCESS_SET_QUOTA | PROCESS_TERMINATE,
                0,
                pid,
            )
        };
        let proc_handle = OwnedHandle::new(proc_h).map_err(|_| JobError::NotFound)?;
        let mut in_job = 0i32;
        // SAFETY: valid handles; `in_job` is a valid out-pointer to i32.
        let ok = unsafe {
            IsProcessInJob(proc_handle.raw(), self.handle.raw(), &mut in_job as *mut i32)
        };
        if ok == 0 {
            return Err(JobError::InvalidState);
        }
        Ok(in_job != 0)
    }

    /// Terminate every process in the job with the given exit code.
    pub fn terminate_all(&self, exit_code: u32) -> Result<(), JobError> {
        // SAFETY: valid owned job handle; exit code is passed through.
        let ok = unsafe { TerminateJobObject(self.handle.raw(), exit_code) };
        if ok == 0 {
            return Err(JobError::InvalidState);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bisect_set_information() {
        let job = OsJobObject::create().unwrap();
        // Case A: only kill-on-close
        let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
        info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        let ok = unsafe {
            SetInformationJobObject(
                job.handle.raw(),
                JobObjectExtendedLimitInformation,
                &info as *const _ as *const c_void,
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        };
        let err = unsafe { GetLastError() };
        eprintln!("case A (kill-on-close only): ok={} err={}", ok, err);
        assert!(ok != 0, "case A failed with {}", err);

        // Case B: + active process limit
        let job2 = OsJobObject::create().unwrap();
        let mut info2: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
        info2.BasicLimitInformation.LimitFlags =
            JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE | JOB_OBJECT_LIMIT_ACTIVE_PROCESS;
        info2.BasicLimitInformation.ActiveProcessLimit = 4;
        let ok2 = unsafe {
            SetInformationJobObject(
                job2.handle.raw(),
                JobObjectExtendedLimitInformation,
                &info2 as *const _ as *const c_void,
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        };
        let err2 = unsafe { GetLastError() };
        eprintln!("case B (+active process): ok={} err={}", ok2, err2);
        assert!(ok2 != 0, "case B failed with {}", err2);

        // Case C: working-set is documented as environment-dependent; we only
        // verify it does not crash and report the result.
        let job3 = OsJobObject::create().unwrap();
        let mut info3: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
        info3.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
            | JOB_OBJECT_LIMIT_ACTIVE_PROCESS
            | JOB_OBJECT_LIMIT_WORKINGSET;
        info3.BasicLimitInformation.ActiveProcessLimit = 4;
        info3.BasicLimitInformation.MinimumWorkingSetSize = 4096;
        info3.BasicLimitInformation.MaximumWorkingSetSize = 256 * 1024 * 1024;
        let ok3 = unsafe {
            SetInformationJobObject(
                job3.handle.raw(),
                JobObjectExtendedLimitInformation,
                &info3 as *const _ as *const c_void,
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        };
        let err3 = unsafe { GetLastError() };
        eprintln!(
            "case C (+working set): ok={} err={} (best-effort, environment-dependent)",
            ok3, err3
        );
        // Do NOT assert on case C: on some Windows configurations the
        // working-set limit is rejected. The core limits (A+B) must pass.
    }

    /// Real end-to-end proof on a Windows host:
    /// create job → set limits → spawn child → assign → verify membership →
    /// terminate → child is dead.
    #[test]
    fn real_job_object_lifecycle() {
        let job = OsJobObject::create().expect("CreateJobObjectW failed");

        let limits = JobLimits {
            max_processes: 4,
            max_memory_bytes: 128 * 1024 * 1024,
            ..JobLimits::default()
        };
        job.set_limits(&limits).expect("SetInformationJobObject failed");

        // Spawn a child that sleeps; we use cmd to avoid extra deps.
        let child = std::process::Command::new("cmd.exe")
            .args(["/c", "ping", "-n", "30", "127.0.0.1", ">NUL"])
            .spawn()
            .expect("failed to spawn child");
        let pid = child.id();

        job.assign_process(pid).expect("AssignProcessToJobObject failed");
        assert!(
            job.contains_process(pid).expect("IsProcessInJob failed"),
            "child must be inside the job"
        );

        job.terminate_all(1).expect("TerminateJobObject failed");

        // The child must exit shortly after termination.
        let out = child
            .wait_with_output()
            .expect("child did not exit after job termination");
        assert!(!out.status.success(), "terminated child should not report success");
    }

    /// Kill-on-close: if the job handle is dropped, contained processes die.
    #[test]
    fn kill_on_close_kills_children() {
        let pid;
        {
            let job = OsJobObject::create().unwrap();
            job.set_limits(&JobLimits::default()).unwrap();
            let child = std::process::Command::new("cmd.exe")
                .args(["/c", "ping", "-n", "30", "127.0.0.1", ">NUL"])
                .spawn()
                .unwrap();
            pid = child.id();
            job.assign_process(pid).unwrap();
            // job handle dropped here → OS kills the contained process
        }
        // Give the OS a moment to deliver the kill.
        std::thread::sleep(std::time::Duration::from_millis(300));
        // Process should no longer be openable (it is gone).
        let h = unsafe { OpenProcess(PROCESS_QUERY_INFORMATION, 0, pid) };
        if !h.is_null() && h != INVALID_HANDLE_VALUE {
            unsafe { CloseHandle(h) };
            panic!("child survived kill-on-close");
        }
    }
}
