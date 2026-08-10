//! Authority: principal registration, attestation issuance, authorization,
//! and revocation.
//!
//! Scope matching follows the Milestone 1 contract in `spec/tagma-sec.md`:
//! Exact and Prefix rules only. Authorization never depends on path secrecy;
//! paths are observable and replayable by design.

use crate::types::{Attestation, Decision, Epoch, Path, PrincipalId, Scope};

pub trait Authority {
    /// Registers a principal from its credentials and returns its identity.
    fn register(&mut self, cred: &[u8]) -> PrincipalId;

    /// Issues an attestation scoped to `scope`, valid until `valid_until`.
    fn issue(
        &mut self,
        principal: PrincipalId,
        scope: Scope,
        valid_until: Epoch,
    ) -> Option<Attestation>;

    /// Decides whether `att` authorizes `path` at `epoch`.
    fn authorize(&self, att: &Attestation, path: &Path, epoch: Epoch) -> Decision;

    /// Revokes the given scope for the principal, effective from `epoch`.
    fn revoke(&mut self, principal: PrincipalId, scope: &Scope, epoch: Epoch);
}
