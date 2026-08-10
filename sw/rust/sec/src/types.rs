//! Shared types for the tagma-sec layer.

use tagma_core::Coord;

/// Identity of a registered principal.
pub type PrincipalId = u64;

/// Monotonic counter that scopes replay protection.
pub type Epoch = u64;

/// Action label. The PoC workflow uses a single action (route update).
pub type Action = u8;

/// The single action of the PoC workflow: a coordination node route update.
pub const ACTION_ROUTE_UPDATE: Action = 1;

/// A scoped path: an ordered sequence of valid Coords.
pub type Path = Vec<Coord>;

/// Authorization decision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    Allow,
    Deny,
}

/// A scope names a target path or a set of paths.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Scope {
    /// Exact match: the target path equals the scope path.
    Exact(Path),
    /// Prefix match: the scope path is a prefix of the target path.
    Prefix(Path),
}

impl Scope {
    /// Returns true when `path` falls inside this scope under the Milestone 1
    /// matching rules (Exact and Prefix only; see spec/tagma-sec.md).
    pub fn matches(&self, path: &Path) -> bool {
        match self {
            Scope::Exact(p) => p == path,
            Scope::Prefix(p) => path.starts_with(p),
        }
    }

    /// A stable key for revocation bookkeeping: a rule marker byte followed
    /// by the coordinate indices of the scope path.
    pub fn key(&self) -> Vec<u16> {
        let mut k = vec![match self {
            Scope::Exact(_) => 0u16,
            Scope::Prefix(_) => 1u16,
        }];
        let path = match self {
            Scope::Exact(p) | Scope::Prefix(p) => p,
        };
        k.extend(path.iter().map(|c| c.index()));
        k
    }
}

/// A capability statement binding a principal to a scope, with a lifetime.
#[derive(Clone, Debug)]
pub struct Attestation {
    pub id: u64,
    pub principal: PrincipalId,
    pub scope: Scope,
    pub issued_epoch: Epoch,
    pub valid_until: Epoch,
}

/// Integrity binding over a record, its path, a principal, and an epoch.
#[derive(Clone, Debug)]
pub struct Seal {
    pub tag: [u8; 32],
}

/// An entry in the append-only audit log.
#[derive(Clone, Debug)]
pub struct Event {
    pub id: u64,
    pub prev: Option<u64>,
    pub payload_hash: [u8; 32],
    pub epoch: Epoch,
}

/// Signed evidence for non-repudiation of origin.
#[derive(Clone, Debug)]
pub struct SignedEvidence {
    pub evidence: Vec<u8>,
    pub tag: [u8; 32],
}

/// Proof of delivery and origin for exchanged evidence.
#[derive(Clone, Debug)]
pub struct Receipt {
    pub signed: SignedEvidence,
    pub remote: PrincipalId,
    pub epoch: Epoch,
}
