//! tagma-sec pattern.
//!
//! The authority matches scopes over `CoordPath` values (Exact and Prefix)
//! and keeps an epoch-scoped revocation list. The integrity seal binds the
//! record, the path, the principal, and the epoch, so replay across epochs
//! is detected.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::authority::Authority;
use crate::integrity::Integrity;
use crate::types::{Action, Attestation, Decision, Epoch, Path, PrincipalId, Scope, Seal};

const DELOS_INTEGRITY_KEY: &[u8; 32] = b"tagma-sec-poc-delos-int-key-0000";

/// CoordPath-scoped authority with epoch-scoped revocation.
pub struct DelosAuthority {
    next_id: u64,
    next_att: u64,
    attestations: BTreeMap<PrincipalId, Vec<Attestation>>,
    revoked: BTreeMap<(PrincipalId, Vec<u16>), Epoch>,
}

impl DelosAuthority {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            next_att: 0,
            attestations: BTreeMap::new(),
            revoked: BTreeMap::new(),
        }
    }
}

impl Default for DelosAuthority {
    fn default() -> Self {
        Self::new()
    }
}

impl Authority for DelosAuthority {
    fn register(&mut self, _cred: &[u8]) -> PrincipalId {
        self.next_id += 1;
        self.next_id
    }

    fn issue(
        &mut self,
        principal: PrincipalId,
        scope: Scope,
        issued_epoch: Epoch,
        valid_until: Epoch,
    ) -> Option<Attestation> {
        self.next_att += 1;
        let att = Attestation {
            id: self.next_att,
            principal,
            scope,
            issued_epoch,
            valid_until,
        };
        self.attestations
            .entry(principal)
            .or_default()
            .push(att.clone());
        Some(att)
    }

    fn authorize(&self, att: &Attestation, path: &Path, _action: Action, epoch: Epoch) -> Decision {
        // The presented attestation carries the holder's identity claim; the
        // window, the scope, and the revocation record are read from the
        // stored attestation, so a presented copy cannot extend or reshape
        // the grant.
        let Some(stored) = self
            .attestations
            .get(&att.principal)
            .and_then(|list| list.iter().find(|a| a.id == att.id))
        else {
            return Decision::Deny;
        };
        if !stored.scope.matches(path) {
            return Decision::Deny;
        }
        if epoch < stored.issued_epoch || epoch > stored.valid_until {
            return Decision::Deny;
        }
        if let Some(&revoked_at) = self.revoked.get(&(att.principal, stored.scope.key())) {
            if epoch >= revoked_at {
                return Decision::Deny;
            }
        }
        Decision::Allow
    }

    fn revoke(&mut self, principal: PrincipalId, scope: &Scope, epoch: Epoch) {
        self.revoked.insert((principal, scope.key()), epoch);
    }
}

/// Integrity seal for the tagma-sec pattern: binds record, path, principal,
/// and epoch.
pub struct DelosIntegrity;

impl Integrity for DelosIntegrity {
    fn seal(&self, record: &[u8], path: &Path, principal: PrincipalId, epoch: Epoch) -> Seal {
        let mut h = blake3::Hasher::new_keyed(DELOS_INTEGRITY_KEY);
        h.update(record);
        h.update(&principal.to_le_bytes());
        h.update(&epoch.to_le_bytes());
        for c in path {
            h.update(&c.index().to_le_bytes());
        }
        Seal {
            tag: h.finalize().into(),
        }
    }

    fn verify(
        &self,
        record: &[u8],
        path: &Path,
        principal: PrincipalId,
        epoch: Epoch,
        seal: &Seal,
    ) -> bool {
        self.seal(record, path, principal, epoch).tag == seal.tag
    }

    fn refresh(
        &self,
        record: &[u8],
        path: &Path,
        principal: PrincipalId,
        from_epoch: Epoch,
        to_epoch: Epoch,
        seal: &Seal,
    ) -> Option<Seal> {
        if to_epoch <= from_epoch {
            return None;
        }
        if self.verify(record, path, principal, from_epoch, seal) {
            Some(self.seal(record, path, principal, to_epoch))
        } else {
            None
        }
    }
}
