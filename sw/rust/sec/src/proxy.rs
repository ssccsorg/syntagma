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
use crate::types::{Attestation, Decision, Epoch, Event, Path, Receipt, ACTION_ROUTE_UPDATE};

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

/// Outcome of a successful route update: the record's audit entry and the
/// receipt proving delivery and origin (non-repudiation).
#[derive(Clone, Debug)]
pub struct RouteUpdate {
    /// Audit entry committing to the record.
    pub event: Event,
    /// Receipt binding the record to its path, epoch, and remote principal.
    pub receipt: Receipt,
}

/// Scenario workflow: a coordination node submits a route update.
///
/// The update is accepted only when the attestation authorizes the path and
/// action at the current epoch and the sealed record verifies. The record is
/// then committed to the audit log, and the channel exchanges a receipt
/// binding the record to its path, epoch, and origin principal; the signed
/// evidence is committed to the audit log as the non-repudiation receipt.
/// Returns `None` on any rejected step.
pub fn route_update(
    stack: &mut SecStack,
    att: &Attestation,
    path: &Path,
    record: &[u8],
    epoch: Epoch,
) -> Option<RouteUpdate> {
    if stack
        .authority
        .authorize(att, path, ACTION_ROUTE_UPDATE, epoch)
        != Decision::Allow
    {
        return None;
    }
    let seal = stack.integrity.seal(record, path, att.principal, epoch);
    if !stack
        .integrity
        .verify(record, path, att.principal, epoch, &seal)
    {
        return None;
    }
    let event = stack.audit.append(record, epoch);

    // Non-repudiation: bind the record to its path and epoch, exchange a
    // receipt with the channel, and commit the signed evidence to the audit
    // log (receipts are appended by the caller per spec/tagma-sec.md).
    let mut evidence = Vec::with_capacity(record.len() + path.len() * 2 + 8);
    evidence.extend_from_slice(record);
    for c in path {
        evidence.extend_from_slice(&c.index().to_le_bytes());
    }
    evidence.extend_from_slice(&epoch.to_le_bytes());
    let receipt = stack.channel.exchange(&evidence, att.principal, epoch);
    stack.audit.append(&receipt.signed.evidence, epoch);

    Some(RouteUpdate { event, receipt })
}
