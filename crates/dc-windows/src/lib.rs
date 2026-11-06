//! C-12 Windows authority-boundary slice: typed identity and a short admission
//! gate.
//!
//! Everything in this crate is deterministic and pure. No Win32 call, handle,
//! token, named pipe, Job Object, AppContainer profile or DPAPI buffer is
//! opened or proven here. The OS-side acquisition adapters (`authenticate
//! pipe peer`, `create_contained_process`, `open_verified_target`,
//! `protect_key`) are explicit future work; this slice supplies the typed
//! checks they must feed with values acquired at the real boundary, and does
//! not claim to prove those values by itself.

pub mod admission;
pub mod appcontainer;
pub mod dpapi;
pub mod identity;
pub mod job;
pub mod job_os;
pub mod named_pipe;
