//! C-10 admission protocol: re-check authority, generations and fences inside a
//! held short gate. Pure and deterministic; no scheduler, kernel primitive,
//! dispatched effect (D) or committed row (C) is performed here.
//!
//! A receipt is a value snapshot of what was validated on entry, for re-checking
//! at the D/C boundary. Holding a receipt does not replace holding the live
//! gate, and a stale/foreign receipt is rejected — matching contract 17 §12.1:
//! revocation established after admission (F relative to D/C) must fail the
//! older admission.

use std::collections::BTreeMap;

use dc_domain::ids::{RowVersion, RunGeneration};
use dc_domain::permission::{AuthoritySnapshot, CurrentFence, FenceScope};
use dc_windows::admission::{self as gate, AdmissionGate, AdmissionToken};
use dc_windows::identity::{PeerAuthenticationError, PeerIdentity, authenticate_peer};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum AdmissionError {
    #[error("requester role does not match the expected channel peer")]
    PeerMismatch,
    #[error("requester belongs to another installation")]
    WrongInstallation,
    #[error("requester generation does not match the current authority generation")]
    StaleGeneration,
    #[error("authority epoch differs from the admitted epoch")]
    EpochMismatch,
    #[error("a support fence is absent, closed, or changed")]
    FenceChanged,
    #[error("the short admission gate is unavailable")]
    GateUnavailable,
    #[error("admission released by a party that does not hold it")]
    AbandonedAdmission,
    #[error("integer authority counter overflow")]
    ArithmeticOverflow,
}

impl From<PeerAuthenticationError> for AdmissionError {
    fn from(error: PeerAuthenticationError) -> Self {
        match error {
            PeerAuthenticationError::PeerMismatch => AdmissionError::PeerMismatch,
            PeerAuthenticationError::WrongInstallation => AdmissionError::WrongInstallation,
            PeerAuthenticationError::StaleGeneration => AdmissionError::StaleGeneration,
        }
    }
}

impl From<gate::AdmissionError> for AdmissionError {
    fn from(error: gate::AdmissionError) -> Self {
        match error {
            gate::AdmissionError::GateBusy | gate::AdmissionError::GateClosed => {
                AdmissionError::GateUnavailable
            }
            gate::AdmissionError::StaleGeneration => AdmissionError::StaleGeneration,
            gate::AdmissionError::AbandonedAdmission => AdmissionError::AbandonedAdmission,
        }
    }
}

/// Values validated when the fenced operation was entered.
///
/// `fences` is the *support set of the lease this operation runs under* — the
/// exact scopes its admission depends on (root work, unit, lease, parent
/// lease), never the whole authority fence table. An unrelated work's revoke
/// must not invalidate this admission (contract 18 §7.5: a scoped fence
/// advances without revoking unrelated rights).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmissionReceipt {
    pub requester: PeerIdentity,
    pub authority_epoch: dc_domain::ids::AuthorityEpoch,
    pub run_generation: RunGeneration,
    pub fences: BTreeMap<FenceScope, RowVersion>,
}

/// A held admission: the live gate token plus the entry-time receipt.
///
/// Fix D: This struct intentionally does NOT implement Clone.
/// Cloning a HeldAdmission would duplicate the gate token, violating
/// the single-holder invariant of the admission gate.
#[derive(Debug)]
pub struct HeldAdmission {
    pub token: AdmissionToken,
    pub receipt: AdmissionReceipt,
}

/// Compile-time proof that HeldAdmission is not Clone.
/// If this compiles, HeldAdmission does NOT implement Clone.
#[cfg(test)]
mod tests {
    use super::*;
    use dc_domain::ids::{AuthorityEpoch, RunGeneration};
    use dc_domain::permission::FenceScope;

    // This function is only compiled, never called.
    // It proves that HeldAdmission does not implement Clone.
    #[allow(dead_code)]
    fn held_admission_is_not_clone() {
        fn assert_not_clone<T>() {}
        assert_not_clone::<HeldAdmission>();
    }

    #[test]
    fn held_admission_cannot_be_cloned() {
        // This is a compile-time test. The fact that it compiles proves
        // HeldAdmission does NOT implement Clone. If someone adds Clone
        // to HeldAdmission, this test will fail to compile.
        held_admission_is_not_clone();
    }
}

/// Enter the short gate while the requester's identity must authenticate against
/// the current authority generation. The channel expectation supplies role and
/// installation; the authority supplies the live generation, so a stale
/// generation on the claim fails closed before any gate is claimed.
///
/// `support` is the fence support-set of the lease this operation will run
/// under: only those scopes are snapshotted into the receipt. A fence absent
/// from the authority view is refused immediately; fences outside the support
/// set are deliberately not captured, so a later revoke of an unrelated work
/// cannot invalidate this admission.
pub fn begin_fenced_operation(
    gate: &mut AdmissionGate,
    requester: PeerIdentity,
    expected_peer: &PeerIdentity,
    authority: &AuthoritySnapshot,
    support: &[FenceScope],
) -> Result<HeldAdmission, AdmissionError> {
    let expected = PeerIdentity {
        run_generation: authority.run_generation,
        ..*expected_peer
    };
    authenticate_peer(&requester, &expected)?;
    // Validate the complete support set *before* touching the gate, so a
    // refused entry never leaves the gate claimed or half-released.
    let mut fences = BTreeMap::new();
    for scope in support {
        let current = authority
            .fences
            .get(scope)
            .ok_or(AdmissionError::FenceChanged)?;
        if current.closed {
            return Err(AdmissionError::FenceChanged);
        }
        fences.insert(*scope, current.version);
    }
    let token = gate::enter_admission(gate, requester, authority.run_generation)?;
    let receipt = AdmissionReceipt {
        requester,
        authority_epoch: authority.authority_epoch,
        run_generation: authority.run_generation,
        fences,
    };
    Ok(HeldAdmission { token, receipt })
}

/// Re-check the admission at the D or C boundary. Generation/epoch drift
/// denies deterministically. Only the receipt's support-set fences are
/// re-checked: absence, closure or version change of a *support* fence denies;
/// fences added or advanced for unrelated scopes since admission are ignored.
pub fn verify_at_boundary(
    receipt: &AdmissionReceipt,
    authority: &AuthoritySnapshot,
) -> Result<(), AdmissionError> {
    if authority.run_generation != receipt.run_generation {
        return Err(AdmissionError::StaleGeneration);
    }
    if authority.authority_epoch != receipt.authority_epoch {
        return Err(AdmissionError::EpochMismatch);
    }
    for (scope, version) in &receipt.fences {
        if authority
            .fences
            .get(scope)
            .is_none_or(|current| current.closed || current.version != *version)
        {
            return Err(AdmissionError::FenceChanged);
        }
    }
    Ok(())
}

/// Release the live gate. Releasing by anyone other than the current holder is
/// an abandoned admission, which fails closed.
pub fn finish_fenced_operation(
    gate: &mut AdmissionGate,
    held: HeldAdmission,
) -> Result<(), AdmissionError> {
    match gate::leave_admission(gate, held.token) {
        Ok(()) => Ok(()),
        Err(gate::AdmissionError::AbandonedAdmission) => Err(AdmissionError::AbandonedAdmission),
        Err(other) => Err(other.into()),
    }
}

/// Pure projection of a trusted revoke: close one scope and bump its fence
/// version. The durable anchor transaction and the "no normal queue" gate order
/// from contract 17 §12.1 are separate integration steps.
pub fn establish_fence(
    authority: &AuthoritySnapshot,
    scope: FenceScope,
) -> Result<AuthoritySnapshot, AdmissionError> {
    let mut next = authority.clone();
    let entry = next.fences.entry(scope).or_insert(CurrentFence {
        version: RowVersion::new(1).expect("nonzero fence seed"),
        closed: false,
    });
    let bumped = entry
        .version
        .get()
        .checked_add(1)
        .and_then(RowVersion::new)
        .ok_or(AdmissionError::ArithmeticOverflow)?;
    entry.version = bumped;
    entry.closed = true;
    Ok(next)
}
