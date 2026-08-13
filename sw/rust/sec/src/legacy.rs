//! Legacy security pattern.
//!
//! The authority is an ACL-style registry of attestations per principal.
//! Revocation removes the scope immediately. The integrity seal binds only
//! the record and the path; the principal and the epoch do not participate.
//! This is the baseline that the tagma-sec pattern is compared against in
//! the shared workflow tests.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::authority::Authority;
use crate::integrity::Integrity;
use crate::types::{Action, Attestation, Decision, Epoch, Path, PrincipalId, Scope, Seal};

const LEGACY_INTEGRITY_KEY: &[u8; 32] = b"tagma-sec-poc-legacy-int-key-000";

/// ACL-style authority for the legacy pattern.
pub struct LegacyAuthority {
    next_id: u64,
    next_att: u64,
    attestations: BTreeMap<PrincipalId, Vec<Attestation>>,
}

impl LegacyAuthority {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            next_att: 0,
            attestations: BTreeMap::new(),
        }
    }
}

impl Default for LegacyAuthority {
    fn default() -> Self {
        Self::new()
    }
}

impl Authority for LegacyAuthority {
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
        let allowed = self.attestations.get(&att.principal).is_some_and(|list| {
            list.iter().any(|a| {
                a.id == att.id
                    && a.scope.matches(path)
                    && epoch >= a.issued_epoch
                    && epoch <= a.valid_until
            })
        });
        if allowed {
            Decision::Allow
        } else {
            Decision::Deny
        }
    }

    fn revoke(&mut self, principal: PrincipalId, scope: &Scope, _epoch: Epoch) {
        if let Some(list) = self.attestations.get_mut(&principal) {
            list.retain(|a| a.scope != *scope);
        }
    }
}

/// Integrity seal for the legacy pattern: binds record and path only.
pub struct LegacyIntegrity;

impl Integrity for LegacyIntegrity {
    fn seal(&self, record: &[u8], path: &Path, _principal: PrincipalId, _epoch: Epoch) -> Seal {
        let mut h = blake3::Hasher::new_keyed(LEGACY_INTEGRITY_KEY);
        h.update(record);
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
        _principal: PrincipalId,
        _epoch: Epoch,
        seal: &Seal,
    ) -> bool {
        self.seal(record, path, 0, 0).tag == seal.tag
    }

    fn refresh(
        &self,
        record: &[u8],
        path: &Path,
        principal: PrincipalId,
        _from_epoch: Epoch,
        to_epoch: Epoch,
        seal: &Seal,
    ) -> Option<Seal> {
        if self.verify(record, path, principal, 0, seal) {
            Some(self.seal(record, path, principal, to_epoch))
        } else {
            None
        }
    }
}
