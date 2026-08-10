//! Channel: non-repudiation of origin and receipt.
//!
//! `MacChannel` is the shared PoC implementation: evidence is signed with a
//! keyed tag and verified by recomputation. A production deployment replaces
//! this with a signature scheme over the module interface.

use alloc::vec::Vec;

use crate::hash::keyed_tag;
use crate::types::{Epoch, PrincipalId, Receipt, SignedEvidence};

pub trait Channel {
    /// Signs evidence, producing a verifiable signed message.
    fn sign(&self, evidence: &[u8]) -> SignedEvidence;

    /// Accepts only signed evidence whose tag matches the evidence.
    fn verify(&self, signed: &SignedEvidence) -> bool;

    /// Produces a receipt binding `local`, `remote`, and `epoch`.
    fn exchange(&self, local: &[u8], remote: PrincipalId, epoch: Epoch) -> Receipt;

    /// Accepts only receipts whose signed evidence matches the remote and
    /// epoch recorded on the receipt.
    fn verify_receipt(&self, receipt: &Receipt) -> bool;
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

    fn exchange(&self, local: &[u8], remote: PrincipalId, epoch: Epoch) -> Receipt {
        let mut evidence = Vec::with_capacity(local.len() + 16);
        evidence.extend_from_slice(local);
        evidence.extend_from_slice(&remote.to_le_bytes());
        evidence.extend_from_slice(&epoch.to_le_bytes());
        Receipt {
            signed: self.sign(&evidence),
            remote,
            epoch,
        }
    }

    fn verify_receipt(&self, receipt: &Receipt) -> bool {
        let n = receipt.signed.evidence.len();
        if n < 16 {
            return false;
        }
        let tail = &receipt.signed.evidence[n - 16..];
        let remote = u64::from_le_bytes(tail[..8].try_into().expect("8 bytes"));
        let epoch = u64::from_le_bytes(tail[8..].try_into().expect("8 bytes"));
        self.verify(&receipt.signed) && remote == receipt.remote && epoch == receipt.epoch
    }
}
