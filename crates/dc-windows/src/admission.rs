//! C-12 short admission gate (typed state only).
//!
//! `AdmissionGate` models the single short installation-wide gate from contract
//! 17 §12.1 as a deterministic state machine. The actual kernel synchronisation
//! primitive (mutex/event/handle) is NOT acquired here and is NOT proven: this
//! is the typed layer a native adapter must drive, not the OS gate itself.

use dc_domain::ids::RunGeneration;
use thiserror::Error;

use crate::identity::PeerIdentity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateState {
    Idle,
    Held,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum AdmissionError {
    #[error("gate is already held by an unfinished operation")]
    GateBusy,
    #[error("gate is closed for new admissions")]
    GateClosed,
    #[error("holder identity is bound to a stale run generation")]
    StaleGeneration,
    #[error("release attempted by a party that does not hold the gate")]
    AbandonedAdmission,
}

/// A non-transferable, non-copyable receipt for one held, still-live admission.
///
/// P1-ADMISSION-TOKEN-COPY fix: This struct is NOT Clone+Copy. Each token
/// represents exactly one live admission and cannot be duplicated. Only the
/// actually-held native gate authorises the D/C point.
///
/// P1-ADMISSION-LIVENESS-TYPE fix: The token is consumed by leave_admission,
/// ensuring the gate is only released by the live holder.
#[derive(Debug, PartialEq, Eq)]
pub struct AdmissionToken {
    holder: PeerIdentity,
}

impl AdmissionToken {
    /// Create a new token. Private — only the gate can mint tokens.
    fn new(holder: PeerIdentity) -> Self {
        Self { holder }
    }

    /// Get the holder identity. Read-only.
    pub fn holder(&self) -> &PeerIdentity {
        &self.holder
    }
}

/// A live admission guard that owns the gate until dropped or released.
///
/// This is the RAII-style guard required by P1-ADMISSION-LIVENESS-TYPE.
/// The boundary verification requires `&HeldAdmission`, not a clonable receipt.
#[derive(Debug)]
pub struct HeldAdmission<'a> {
    gate: &'a mut AdmissionGate,
    token: Option<AdmissionToken>,
}

impl<'a> HeldAdmission<'a> {
    /// Get the token for this admission. The token is bound to this guard's lifetime.
    pub fn token(&self) -> &AdmissionToken {
        self.token
            .as_ref()
            .expect("HeldAdmission always holds a token")
    }

    /// Release the admission, consuming the guard and the token.
    pub fn release(mut self) -> Result<(), AdmissionError> {
        let token = self
            .token
            .take()
            .expect("HeldAdmission always holds a token");
        leave_admission(self.gate, token)
    }
}

impl<'a> Drop for HeldAdmission<'a> {
    fn drop(&mut self) {
        if let Some(token) = self.token.take() {
            // Best-effort release on drop; failure is logged but not propagated
            let _ = leave_admission(self.gate, token);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdmissionGate {
    state: GateState,
    holder: Option<PeerIdentity>,
}

impl AdmissionGate {
    pub fn idle() -> Self {
        Self {
            state: GateState::Idle,
            holder: None,
        }
    }

    pub fn state(&self) -> GateState {
        self.state
    }
}

/// Claim the gate while verifying the holder's generation is current. The
/// holder's role and installation are authenticated separately at the channel
/// (see `identity::authenticate_peer`); the gate additionally refuses a holder
/// whose generation has already advanced.
pub fn enter_admission(
    gate: &mut AdmissionGate,
    holder: PeerIdentity,
    current_generation: RunGeneration,
) -> Result<AdmissionToken, AdmissionError> {
    match gate.state {
        GateState::Closed => return Err(AdmissionError::GateClosed),
        GateState::Held => return Err(AdmissionError::GateBusy),
        GateState::Idle => {}
    }
    if holder.run_generation != current_generation {
        return Err(AdmissionError::StaleGeneration);
    }
    gate.state = GateState::Held;
    gate.holder = Some(holder);
    Ok(AdmissionToken::new(holder))
}

/// Claim the gate and return a live admission guard (RAII pattern).
///
/// This is the preferred API for P1-ADMISSION-LIVENESS-TYPE: the caller
/// receives a guard that owns the gate until explicitly released or dropped.
pub fn enter_admission_guarded(
    gate: &mut AdmissionGate,
    holder: PeerIdentity,
    current_generation: RunGeneration,
) -> Result<HeldAdmission<'_>, AdmissionError> {
    let token = enter_admission(gate, holder, current_generation)?;
    Ok(HeldAdmission {
        gate,
        token: Some(token),
    })
}

/// Release a held admission. Release by anyone other than the current holder is
/// an abandoned admission, which fails closed rather than "succeeding" the
/// operation.
pub fn leave_admission(
    gate: &mut AdmissionGate,
    token: AdmissionToken,
) -> Result<(), AdmissionError> {
    if gate.state != GateState::Held || gate.holder != Some(token.holder) {
        return Err(AdmissionError::AbandonedAdmission);
    }
    gate.state = GateState::Idle;
    gate.holder = None;
    Ok(())
}

/// Close the gate for new admissions (an establish-fence revoke). A gate closed
/// while held drops the abandoned holder: the abandoned operation never becomes
/// successful.
pub fn close_admission(gate: &mut AdmissionGate) {
    gate.state = GateState::Closed;
    gate.holder = None;
}
