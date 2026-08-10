//! Proxy: composes the four modules behind trait objects and switches
//! between the legacy and tagma-sec stacks. The scenario workflow is written
//! only against the trait objects, so the same requirements and outputs are
//! exercised against both implementations.

use crate::audit::{Audit, ChainedAudit};
use crate::authority::Authority;
use crate::channel::{Channel, MacChannel};
use crate::delos::{DelosAuthority, DelosIntegrity};
use crate::integrity::Integrity;
use crate::legacy::{LegacyAuthority, LegacyIntegrity};
use crate::types::{Attestation, Decision, Epoch, Event, Path};

/// A composable security stack with switchable implementations.
pub struct SecStack {
    pub authority: Box<dyn Authority>,
    pub integrity: Box<dyn Integrity>,
    pub audit: Box<dyn Audit>,
    pub channel: Box<dyn Channel>,
}

impl SecStack {
    /// The legacy security pattern stack.
    pub fn legacy() -> Self {
        Self {
            authority: Box::new(LegacyAuthority::new()),
            integrity: Box::new(LegacyIntegrity),
            audit: Box::new(ChainedAudit::new()),
            channel: Box::new(MacChannel),
        }
    }

    /// The tagma-sec pattern stack.
    pub fn delos() -> Self {
        Self {
            authority: Box::new(DelosAuthority::new()),
            integrity: Box::new(DelosIntegrity),
            audit: Box::new(ChainedAudit::new()),
            channel: Box::new(MacChannel),
        }
    }
}

/// Scenario workflow: a coordination node submits a route update.
///
/// The update is accepted only when the attestation authorizes the path at
/// the current epoch, the sealed record verifies, and the event is appended
/// to the audit log. Returns the audit event on success, or `None` on any
/// rejected step.
pub fn route_update(
    stack: &mut SecStack,
    att: &Attestation,
    path: &Path,
    record: &[u8],
    epoch: Epoch,
) -> Option<Event> {
    if stack.authority.authorize(att, path, epoch) != Decision::Allow {
        return None;
    }
    let seal = stack.integrity.seal(record, path, att.principal, epoch);
    if !stack
        .integrity
        .verify(record, path, att.principal, epoch, &seal)
    {
        return None;
    }
    Some(stack.audit.append(record, epoch))
}
