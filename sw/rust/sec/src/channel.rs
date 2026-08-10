//! Channel: non-repudiation of origin and receipt.
//!
//! `MacChannel` is the shared PoC implementation: evidence is signed with a
//! keyed tag and verified by recomputation. A production deployment replaces
//! this with a signature scheme over the module interface.

use crate::hash::keyed_tag;
use crate::types::SignedEvidence;

pub trait Channel {
    /// Signs evidence, producing a verifiable signed message.
    fn sign(&self, evidence: &[u8]) -> SignedEvidence;

    /// Accepts only signed evidence whose tag matches the evidence.
    fn verify(&self, signed: &SignedEvidence) -> bool;
}

const CHANNEL_KEY: &[u8; 32] = b"tagma-sec-poc-channel-key-000000";

/// Keyed-tag channel shared by the legacy and tagma-sec stacks.
pub struct MacChannel;

impl Channel for MacChannel {
    fn sign(&self, evidence: &[u8]) -> SignedEvidence {
        SignedEvidence {
            evidence: evidence.to_vec(),
            tag: keyed_tag(CHANNEL_KEY, &[evidence]),
        }
    }

    fn verify(&self, signed: &SignedEvidence) -> bool {
        keyed_tag(CHANNEL_KEY, &[&signed.evidence]) == signed.tag
    }
}
