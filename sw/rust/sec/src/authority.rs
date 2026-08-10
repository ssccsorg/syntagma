//! Authority: principal registration, attestation issuance, authorization,
//! and revocation.
//!
//! Scope matching follows the Milestone 1 contract in `spec/tagma-sec.md`:
//! Exact and Prefix rules only. Authorization never depends on path secrecy;
//! paths are observable and replayable by design.

use crate::types::{Action, Attestation, Decision, Epoch, Path, PrincipalId, Scope};

pub trait Authority {
    /// Registers a principal from its credentials and returns its identity.
    fn register(&mut self, cred: &[u8]) -> PrincipalId;

    /// Issues an attestation scoped to `scope`, valid from `issued_epoch`
    /// through `valid_until`.
    fn issue(
        &mut self,
        principal: PrincipalId,
        scope: Scope,
        issued_epoch: Epoch,
        valid_until: Epoch,
    ) -> Option<Attestation>;

    /// Decides whether `att` authorizes `path` for `action` at `epoch`.
    ///
    /// The action is carried by the interface; the Milestone 1 policy is
    /// scope-only, so implementations do not yet discriminate by action.
    fn authorize(&self, att: &Attestation, path: &Path, action: Action, epoch: Epoch) -> Decision;

    /// Revokes the given scope for the principal, effective from `epoch`.
    fn revoke(&mut self, principal: PrincipalId, scope: &Scope, epoch: Epoch);
}
