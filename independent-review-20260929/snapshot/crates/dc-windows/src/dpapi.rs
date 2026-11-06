//! Real DPAPI encryption for sealed store keys.
//!
//! This module performs the actual Win32 CryptProtectData/CryptUnprotectData
//! calls to seal and unseal the SQLite store key. The sealed key is tied to
//! the current Windows user account and cannot be decrypted by other users
//! or after OS reinstall.
//!
//! Safety model: `unsafe_code` is denied workspace-wide. The ONLY unsafe
//! blocks in this module are explicitly allowed and justified.

#![cfg(windows)]
#![allow(unsafe_code)]

use std::ffi::c_void;

use windows_sys::Win32::Security::Cryptography::{
    CryptProtectData, CryptUnprotectData, CRYPT_INTEGER_BLOB,
};

/// Error type for DPAPI operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DpapiError {
    /// Encryption failed
    ProtectFailed,
    /// Decryption failed
    UnprotectFailed,
    /// Invalid input (empty or malformed)
    InvalidInput,
}

impl std::fmt::Display for DpapiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DpapiError::ProtectFailed => write!(f, "CryptProtectData failed"),
            DpapiError::UnprotectFailed => write!(f, "CryptUnprotectData failed"),
            DpapiError::InvalidInput => write!(f, "invalid input for DPAPI operation"),
        }
    }
}

impl std::error::Error for DpapiError {}

/// A DPAPI-sealed blob: encrypted with the current user's credentials.
/// The blob can be stored on disk; only the same Windows user can decrypt it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SealedBlob {
    data: Vec<u8>,
}

impl SealedBlob {
    /// Get the raw encrypted bytes for storage.
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Reconstruct from stored bytes (does NOT decrypt).
    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self { data }
    }

    /// Get the length of the sealed data.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the sealed data is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// Seal (encrypt) data using the current Windows user's DPAPI key.
///
/// The encrypted blob is tied to the current user account and machine.
/// Only the same user on the same machine can decrypt it.
pub fn seal(data: &[u8]) -> Result<SealedBlob, DpapiError> {
    if data.is_empty() {
        return Err(DpapiError::InvalidInput);
    }

    let mut in_blob = CRYPT_INTEGER_BLOB {
        cbData: data.len() as u32,
        pbData: data.as_ptr() as *mut u8,
    };

    let mut out_blob = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: std::ptr::null_mut(),
    };

    // SAFETY: All pointers are valid; in_blob points to caller's data,
    // out_blob is allocated by CryptProtectData and freed by LocalFree.
    let ok = unsafe {
        CryptProtectData(
            &mut in_blob,
            std::ptr::null(), // no description
            std::ptr::null(), // no entropy
            std::ptr::null_mut(), // no reserved
            std::ptr::null(), // no prompt struct
            0, // no flags
            &mut out_blob,
        )
    };

    if ok == 0 {
        return Err(DpapiError::ProtectFailed);
    }

    // Copy the encrypted data into a Rust Vec before freeing.
    let sealed = unsafe {
        std::slice::from_raw_parts(out_blob.pbData, out_blob.cbData as usize).to_vec()
    };

    // Free the DPAPI-allocated buffer.
    // LocalFree is in Win32::Foundation, not System::Memory.
    unsafe {
        windows_sys::Win32::Foundation::LocalFree(out_blob.pbData as *mut c_void);
    }

    Ok(SealedBlob { data: sealed })
}

/// Unseal (decrypt) a DPAPI-sealed blob using the current Windows user's key.
///
/// Returns the original plaintext data. Fails if the blob was encrypted
/// by a different user or on a different machine.
pub fn unseal(blob: &SealedBlob) -> Result<Vec<u8>, DpapiError> {
    if blob.is_empty() {
        return Err(DpapiError::InvalidInput);
    }

    let mut in_blob = CRYPT_INTEGER_BLOB {
        cbData: blob.data.len() as u32,
        pbData: blob.data.as_ptr() as *mut u8,
    };

    let mut out_blob = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: std::ptr::null_mut(),
    };

    // SAFETY: All pointers are valid; in_blob points to caller's data,
    // out_blob is allocated by CryptUnprotectData and freed by LocalFree.
    let ok = unsafe {
        CryptUnprotectData(
            &mut in_blob,
            std::ptr::null_mut(), // no description
            std::ptr::null(), // no entropy
            std::ptr::null_mut(), // no reserved
            std::ptr::null(), // no prompt struct
            0, // no flags
            &mut out_blob,
        )
    };

    if ok == 0 {
        return Err(DpapiError::UnprotectFailed);
    }

    let plaintext = unsafe {
        std::slice::from_raw_parts(out_blob.pbData, out_blob.cbData as usize).to_vec()
    };

    unsafe {
        windows_sys::Win32::Foundation::LocalFree(out_blob.pbData as *mut c_void);
    }

    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dpapi_roundtrip() {
        let original = b"deepcreate-store-key-v1";
        let sealed = seal(original).expect("seal failed");
        assert!(!sealed.is_empty());
        assert_ne!(sealed.as_bytes(), original.as_slice());

        let unsealed = unseal(&sealed).expect("unseal failed");
        assert_eq!(unsealed, original);
    }

    #[test]
    fn dpapi_empty_input_rejected() {
        assert_eq!(seal(b""), Err(DpapiError::InvalidInput));
        assert_eq!(unseal(&SealedBlob::from_bytes(vec![])), Err(DpapiError::InvalidInput));
    }
}
