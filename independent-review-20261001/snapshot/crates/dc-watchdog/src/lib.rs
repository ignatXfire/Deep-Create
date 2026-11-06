//! C-10 watchdog authority-boundary slice: authority epoch/generation, the
//! admission fence protocol over a short native gate, and the recovery evidence
//! handling for external-action outcomes.
//!
//! Pure and deterministic. `authority.anchor` persistence, the anti-rollback
//! anchor, job-tree containment, real process termination and the live
//! reconciliation handler are NOT proven here; they remain the work of their
//! C-10/C-12/C-09 integration steps. This slice only supplies the typed
//! epoch/generation arithmetic, the admission re-check logic and the
//! classify/reconcile-or-refuse transitions those steps must drive through the
//! live Windows gate and the durable anchor.

pub mod admission;
pub mod anchor_store;
pub mod authority_anchor;
pub mod recovery;
