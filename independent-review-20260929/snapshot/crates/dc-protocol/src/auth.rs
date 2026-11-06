//! C-02 authenticated sender layer.
//! 
//! After decode produces a candidate Envelope, this module provides
//! sender authentication. No I/O, no network — pure cryptographic verification.

use dc_domain::ids::EntityId;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{Envelope, ProtocolContext, ProtocolError};

/// Authentication errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum AuthError {
    #[error("unknown sender identity")]
    UnknownSender,
    #[error("invalid signature or proof")]
    InvalidProof,
    #[error("sender not authorized for this command")]
    NotAuthorized,
    #[error("authentication expired")]
    Expired,
    #[error("nonce reuse detected")]
    NonceReuse,
    #[error("proof timestamp arithmetic overflow")]
    TimestampOverflow,
    #[error("proof timestamp is too far in the future")]
    FutureSkew,
}

/// Maximum allowed future skew for proof timestamps (logical units).
/// A proof timestamped far ahead of current_time indicates a misconfigured
/// or adversarial sender; refusing it prevents bypass of the expiry check.
pub const MAX_FUTURE_SKEW: u64 = 300_000;

/// Proof of sender identity. Opaque to C-02 — verified by C-12.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SenderProof {
    /// The claimed sender identity.
    pub sender_id: EntityId,
    /// Cryptographic proof bytes (signature, MAC, etc.).
    pub proof_bytes: Vec<u8>,
    /// Nonce to prevent replay.
    pub nonce: u64,
    /// Timestamp or logical clock value.
    pub timestamp: u64,
}

/// An envelope whose sender proof has been accepted by a [`ProofVerifier`].
///
/// This is a trust-bearing type: the fields are private and the ONLY
/// constructor is [`authenticate_envelope`]. Downstream code cannot fabricate
/// one from an arbitrary envelope plus proof bytes — the "authenticated" in
/// the name is enforced by visibility, not by convention. It is consumed by
/// [`validate_authenticated`], which applies the closed C-02 role table and
/// replay semantics; there is no parallel path from this type to the state
/// machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthenticatedEnvelope {
    inner: super::DecodedEnvelope,
    proof: SenderProof,
}

impl AuthenticatedEnvelope {
    /// The decoded envelope (parsed candidate + its exact wire bytes).
    pub fn decoded(&self) -> &super::DecodedEnvelope {
        &self.inner
    }

    /// The verified sender proof.
    pub fn proof(&self) -> &SenderProof {
        &self.proof
    }
}

/// Verifier trait for sender proofs. Implemented by C-12.
/// C-02 defines the interface; C-12 provides the cryptography.
pub trait ProofVerifier: Send + Sync {
    /// Verify a sender proof for a given envelope.
    /// Returns Ok(authenticated_sender_id) on success.
    fn verify(
        &self,
        envelope: &Envelope,
        proof: &SenderProof,
        context: &ProtocolContext,
    ) -> Result<EntityId, AuthError>;
    
    /// Check if a nonce has been used before.
    fn check_nonce(&self, sender_id: &EntityId, nonce: u64) -> Result<(), AuthError>;
}

/// Authentication context — what C-02 needs to verify a sender.
#[derive(Debug, Clone)]
pub struct AuthContext {
    /// Protocol context (installation, run generation).
    pub protocol: ProtocolContext,
    /// Maximum age for authentication proofs.
    pub max_proof_age: u64,
    /// Current logical time for expiration checks.
    pub current_time: u64,
}

/// Authenticate a candidate envelope with a sender proof.
/// This is the C-02 boundary: after this, the envelope is authenticated.
/// Before this, it's just a candidate.
///
/// Takes the [`super::DecodedEnvelope`] produced by [`super::decode_envelope`]
/// so the verified proof is bound to the exact wire bytes, not to a detached
/// parsed value the caller could have paired with anything.
pub fn authenticate_envelope<V: ProofVerifier>(
    envelope: super::DecodedEnvelope,
    proof: SenderProof,
    verifier: &V,
    context: &AuthContext,
) -> Result<AuthenticatedEnvelope, AuthError> {
    // Reject proofs timestamped implausibly far in the future (misconfigured
    // or adversarial senders), before the expiry arithmetic runs.
    if proof.timestamp > context.current_time.saturating_add(MAX_FUTURE_SKEW) {
        return Err(AuthError::FutureSkew);
    }
    // Check proof expiration using checked arithmetic: a u64 overflow must not
    // silently wrap the deadline and revive an expired proof.
    let deadline = proof
        .timestamp
        .checked_add(context.max_proof_age)
        .ok_or(AuthError::TimestampOverflow)?;
    if deadline < context.current_time {
        return Err(AuthError::Expired);
    }

    // Check nonce
    verifier.check_nonce(&proof.sender_id, proof.nonce)?;

    // Verify the proof against the parsed envelope
    let authenticated_sender = verifier.verify(envelope.envelope(), &proof, &context.protocol)?;

    // Verify the claimed sender matches the authenticated sender
    if authenticated_sender != proof.sender_id {
        return Err(AuthError::UnknownSender);
    }

    Ok(AuthenticatedEnvelope {
        inner: envelope,
        proof,
    })
}

/// Full C-02 pipeline for an authenticated wire message: proof verification
/// (already done, sealed in `authenticated`) followed by the closed
/// role-table authority decision and replay semantics.
///
/// This is the unification point: there is exactly ONE path from wire bytes
/// to [`super::SenderValidation`]. `peer` is the channel identity C-12
/// attested for the connection that carried this envelope; the proof's
/// `sender_id` must match it, so a proof verified for one peer cannot be
/// replayed to claim another channel's role.
pub fn validate_authenticated(
    authenticated: AuthenticatedEnvelope,
    peer: &super::AuthenticatedPeer,
    table: &super::RoleTable,
    replay: &mut super::ReplayCache,
    context: &AuthContext,
) -> Result<super::SenderValidation, super::ValidateSenderError> {
    if authenticated.proof.sender_id != peer.peer_id() {
        // Proof belongs to a different identity than the attested channel:
        // refuse as a role/identity failure, never reach the replay cache.
        return Err(super::ValidateSenderError::WrongRole);
    }
    super::validate_sender(authenticated.inner, peer, &context.protocol, table, replay)
}

/// Wire format for authenticated envelope.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireAuthenticatedEnvelope {
    schema_version: u32,
    envelope: super::WireEnvelope,
    sender_id: EntityId,
    proof_bytes: Vec<u8>,
    nonce: u64,
    timestamp: u64,
}

/// Decode an authenticated envelope from bytes.
/// This produces a candidate with proof attached — still needs verification.
///
/// Returns the [`super::DecodedEnvelope`] (parsed + exact inner wire bytes
/// bound together) plus the unverified [`SenderProof`]. The decoded pair is
/// what [`authenticate_envelope`] consumes, so the proof check and the later
/// replay fingerprint both see the same bytes.
pub fn decode_authenticated(
    bytes: &[u8],
    context: &ProtocolContext,
) -> Result<(super::DecodedEnvelope, SenderProof), ProtocolError> {
    if bytes.len() > super::MAX_ENVELOPE_BYTES {
        return Err(ProtocolError::Oversize);
    }

    let wire: WireAuthenticatedEnvelope = serde_json::from_slice(bytes)
        .map_err(super::decode_error)?;

    if wire.schema_version != super::PROTOCOL_VERSION {
        return Err(ProtocolError::UnsupportedVersion);
    }

    let envelope = super::decode_envelope(
        &serde_json::to_vec(&wire.envelope).map_err(|_| ProtocolError::InvalidStructure)?,
        context,
    )?;

    let proof = SenderProof {
        sender_id: wire.sender_id,
        proof_bytes: wire.proof_bytes,
        nonce: wire.nonce,
        timestamp: wire.timestamp,
    };

    Ok((envelope, proof))
}

/// Encode an authenticated envelope to bytes.
/// The proof is attached but not yet verified. Encoding never grants
/// authority.
pub fn encode_authenticated(
    envelope: &Envelope,
    proof: &SenderProof,
    context: &ProtocolContext,
) -> Result<Vec<u8>, ProtocolError> {
    let inner_bytes = super::encode_envelope(envelope, context)?;
    let inner_wire: super::WireEnvelope = serde_json::from_slice(&inner_bytes)
        .map_err(|_| ProtocolError::InvalidStructure)?;
    
    let wire = WireAuthenticatedEnvelope {
        schema_version: super::PROTOCOL_VERSION,
        envelope: inner_wire,
        sender_id: proof.sender_id,
        proof_bytes: proof.proof_bytes.clone(),
        nonce: proof.nonce,
        timestamp: proof.timestamp,
    };
    
    let bytes = serde_json::to_vec(&wire).map_err(|_| ProtocolError::InvalidStructure)?;
    if bytes.len() > super::MAX_ENVELOPE_BYTES {
        return Err(ProtocolError::Oversize);
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dc_domain::work::{ExpectedLifecycleBasis, UnitAttemptCommand, UnitRef, WorkScope};
    use std::num::{NonZeroU32, NonZeroU64};

    fn test_id(seed: u128) -> EntityId {
        let base = 0x017f_22e2_79b0_7cc3_98c4_dc0c_0c07_398f_u128;
        EntityId::from_u128(base + seed).expect("valid UUIDv7")
    }

    fn test_context() -> ProtocolContext {
        ProtocolContext {
            installation_id: test_id(100),
            run_generation: NonZeroU64::MIN,
        }
    }

    fn test_envelope() -> crate::DecodedEnvelope {
        let envelope = Envelope {
            command_id: test_id(1),
            installation_id: test_id(100),
            expected: ExpectedLifecycleBasis {
                scope: WorkScope {
                    session_id: test_id(10),
                    work_id: test_id(11),
                    objective_id: test_id(12),
                    objective_revision: NonZeroU32::MIN,
                },
                unit: UnitRef {
                    unit_id: test_id(13),
                    revision: NonZeroU32::MIN,
                },
                unit_row_version: NonZeroU64::MIN,
                attempt_id: None,
                attempt_row_version: None,
                run_generation: NonZeroU64::MIN,
            },
            command: UnitAttemptCommand::Start {},
        };
        // Round-trip through the real codec so the envelope is bound to its
        // exact wire bytes, exactly as production decode delivers it.
        let bytes = super::super::encode_envelope(&envelope, &test_context()).unwrap();
        super::super::decode_envelope(&bytes, &test_context()).unwrap()
    }

    struct TestVerifier {
        valid_sender: EntityId,
    }

    impl ProofVerifier for TestVerifier {
        fn verify(
            &self,
            _envelope: &Envelope,
            proof: &SenderProof,
            _context: &ProtocolContext,
        ) -> Result<EntityId, AuthError> {
            if proof.sender_id == self.valid_sender && proof.proof_bytes == b"valid" {
                Ok(proof.sender_id)
            } else {
                Err(AuthError::InvalidProof)
            }
        }

        fn check_nonce(&self, _sender_id: &EntityId, nonce: u64) -> Result<(), AuthError> {
            if nonce == 999 {
                Err(AuthError::NonceReuse)
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn authenticate_valid_proof() {
        let envelope = test_envelope();
        let sender_id = test_id(200);
        let proof = SenderProof {
            sender_id,
            proof_bytes: b"valid".to_vec(),
            nonce: 1,
            timestamp: 1000,
        };
        let verifier = TestVerifier { valid_sender: sender_id };
        let context = AuthContext {
            protocol: test_context(),
            max_proof_age: 100,
            current_time: 1000,
        };

        let result = authenticate_envelope(envelope, proof, &verifier, &context);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().proof().sender_id, sender_id);
    }

    #[test]
    fn reject_expired_proof() {
        let envelope = test_envelope();
        let sender_id = test_id(200);
        let proof = SenderProof {
            sender_id,
            proof_bytes: b"valid".to_vec(),
            nonce: 1,
            timestamp: 100, // Old
        };
        let verifier = TestVerifier { valid_sender: sender_id };
        let context = AuthContext {
            protocol: test_context(),
            max_proof_age: 100,
            current_time: 1000, // Now is much later
        };

        let result = authenticate_envelope(envelope, proof, &verifier, &context);
        assert_eq!(result, Err(AuthError::Expired));
    }

    #[test]
    fn reject_nonce_reuse() {
        let envelope = test_envelope();
        let sender_id = test_id(200);
        let proof = SenderProof {
            sender_id,
            proof_bytes: b"valid".to_vec(),
            nonce: 999, // Reused
            timestamp: 1000,
        };
        let verifier = TestVerifier { valid_sender: sender_id };
        let context = AuthContext {
            protocol: test_context(),
            max_proof_age: 100,
            current_time: 1000,
        };

        let result = authenticate_envelope(envelope, proof, &verifier, &context);
        assert_eq!(result, Err(AuthError::NonceReuse));
    }

    #[test]
    fn reject_invalid_proof() {
        let envelope = test_envelope();
        let sender_id = test_id(200);
        let proof = SenderProof {
            sender_id,
            proof_bytes: b"invalid".to_vec(), // Wrong
            nonce: 1,
            timestamp: 1000,
        };
        let verifier = TestVerifier { valid_sender: sender_id };
        let context = AuthContext {
            protocol: test_context(),
            max_proof_age: 100,
            current_time: 1000,
        };

        let result = authenticate_envelope(envelope, proof, &verifier, &context);
        assert_eq!(result, Err(AuthError::InvalidProof));
    }

    #[test]
    fn reject_timestamp_overflow() {
        let envelope = test_envelope();
        let sender_id = test_id(200);
        let proof = SenderProof {
            sender_id,
            proof_bytes: b"valid".to_vec(),
            nonce: 1,
            timestamp: u64::MAX, // timestamp + max_proof_age would overflow
        };
        let verifier = TestVerifier { valid_sender: sender_id };
        let context = AuthContext {
            protocol: test_context(),
            max_proof_age: 100,
            current_time: 1000,
        };

        // u64::MAX also trips the future-skew guard first — either way it must
        // not be admitted. Use a near-max value that passes the skew bound to
        // exercise the checked_add path specifically.
        let result = authenticate_envelope(envelope, proof, &verifier, &context);
        assert!(matches!(
            result,
            Err(AuthError::TimestampOverflow) | Err(AuthError::FutureSkew)
        ));

        // Dedicated checked_add overflow: timestamp just under skew boundary
        // but with a max_proof_age that overflows u64 when added.
        let envelope = test_envelope();
        let skew_boundary = 1000u64.saturating_add(MAX_FUTURE_SKEW);
        let proof = SenderProof {
            sender_id,
            proof_bytes: b"valid".to_vec(),
            nonce: 1,
            timestamp: skew_boundary, // passes future-skew (== boundary)
        };
        let context = AuthContext {
            protocol: test_context(),
            max_proof_age: u64::MAX, // skew_boundary + u64::MAX overflows
            current_time: 1000,
        };
        let result = authenticate_envelope(envelope, proof, &verifier, &context);
        assert_eq!(result, Err(AuthError::TimestampOverflow));
    }

    #[test]
    fn reject_future_skewed_proof() {
        let envelope = test_envelope();
        let sender_id = test_id(200);
        let proof = SenderProof {
            sender_id,
            proof_bytes: b"valid".to_vec(),
            nonce: 1,
            // Far in the future relative to current_time — beyond MAX_FUTURE_SKEW.
            timestamp: 1000 + MAX_FUTURE_SKEW + 1,
        };
        let verifier = TestVerifier { valid_sender: sender_id };
        let context = AuthContext {
            protocol: test_context(),
            max_proof_age: 100,
            current_time: 1000,
        };

        let result = authenticate_envelope(envelope, proof, &verifier, &context);
        assert_eq!(result, Err(AuthError::FutureSkew));
    }

    #[test]
    fn accept_proof_at_future_skew_boundary() {
        let envelope = test_envelope();
        let sender_id = test_id(200);
        let proof = SenderProof {
            sender_id,
            proof_bytes: b"valid".to_vec(),
            nonce: 1,
            timestamp: 1000 + MAX_FUTURE_SKEW, // exactly at the boundary — allowed
        };
        let verifier = TestVerifier { valid_sender: sender_id };
        let context = AuthContext {
            protocol: test_context(),
            max_proof_age: 100,
            current_time: 1000,
        };

        let result = authenticate_envelope(envelope, proof, &verifier, &context);
        assert!(result.is_ok());
    }

    #[test]
    fn encode_decode_roundtrip() {
        let decoded = test_envelope();
        let proof = SenderProof {
            sender_id: test_id(200),
            proof_bytes: b"proof".to_vec(),
            nonce: 42,
            timestamp: 12345,
        };
        let context = test_context();

        let bytes = encode_authenticated(decoded.envelope(), &proof, &context).unwrap();
        let (decoded_env, decoded_proof) = decode_authenticated(&bytes, &context).unwrap();

        assert_eq!(decoded_env.envelope().command_id, decoded.envelope().command_id);
        assert_eq!(decoded_proof.sender_id, proof.sender_id);
        assert_eq!(decoded_proof.nonce, proof.nonce);
    }
}
