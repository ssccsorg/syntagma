//! Audit: append-only chained evidence log.
//!
//! `ChainedAudit` is the shared implementation for both patterns: each entry
//! stores the previous entry id, so retroactive modification of an interior
//! entry breaks the chain. Inclusion proofs and evidence bundles pair each
//! event with its payload so an external party can verify commitments
//! without access to the log.

use alloc::vec::Vec;

use crate::types::{Epoch, Event};

pub trait Audit {
    /// Appends an event for `payload` at `epoch` and returns the entry.
    fn append(&mut self, payload: &[u8], epoch: Epoch) -> Event;

    /// Returns true when the chain is intact over entries `from..=to`.
    fn verify_chain(&self, from: u64, to: u64) -> bool;

    /// Returns an inclusion proof for `entry`, or `None` when out of range.
    fn prove(&self, entry: u64) -> Option<InclusionProof>;

    /// Returns an exportable evidence bundle over entries `from..=to`.
    fn export(&self, from: u64, to: u64) -> Option<EvidenceBundle>;
}

/// An event paired with the payload it commits to.
#[derive(Clone, Debug)]
pub struct Entry {
    pub event: Event,
    pub payload: Vec<u8>,
}

/// Chain segment from genesis through a target entry, verifiable without the
/// log.
#[derive(Clone, Debug)]
pub struct InclusionProof {
    pub entries: Vec<Entry>,
}

impl InclusionProof {
    /// Verifies ids 0..=n, prev links, and payload commitments.
    pub fn verify(&self) -> bool {
        verify_entries(&self.entries)
            && self.entries.first().is_some_and(|e| e.event.prev.is_none())
            && self
                .entries
                .iter()
                .enumerate()
                .all(|(i, e)| e.event.id == i as u64)
    }
}

/// Exportable evidence bundle over a range of entries.
#[derive(Clone, Debug)]
pub struct EvidenceBundle {
    pub entries: Vec<Entry>,
}

impl EvidenceBundle {
    /// Verifies internal chain consistency and payload commitments.
    pub fn verify(&self) -> bool {
        verify_entries(&self.entries)
    }
}

fn verify_entries(entries: &[Entry]) -> bool {
    entries.iter().enumerate().all(|(i, e)| {
        blake3::hash(&e.payload).as_bytes() == &e.event.payload_hash
            && (i == 0 || e.event.prev == Some(entries[i - 1].event.id))
    })
}

/// Chained append-only log shared by the legacy and tagma-sec stacks.
pub struct ChainedAudit {
    events: Vec<Event>,
    payloads: Vec<Vec<u8>>,
}

impl ChainedAudit {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            payloads: Vec::new(),
        }
    }
}

impl Default for ChainedAudit {
    fn default() -> Self {
        Self::new()
    }
}

impl Audit for ChainedAudit {
    fn append(&mut self, payload: &[u8], epoch: Epoch) -> Event {
        let prev = self.events.last().map(|e| e.id);
        let id = self.events.len() as u64;
        let event = Event {
            id,
            prev,
            payload_hash: blake3::hash(payload).into(),
            epoch,
        };
        self.events.push(event.clone());
        self.payloads.push(payload.to_vec());
        event
    }

    fn verify_chain(&self, from: u64, to: u64) -> bool {
        let from = from as usize;
        let to = to as usize;
        if from > to || to >= self.events.len() {
            return false;
        }
        for i in from..=to {
            if i == 0 {
                if self.events[i].prev.is_some() {
                    return false;
                }
            } else if self.events[i].prev != Some(self.events[i - 1].id) {
                return false;
            }
        }
        true
    }

    fn prove(&self, entry: u64) -> Option<InclusionProof> {
        let entry = entry as usize;
        if entry >= self.events.len() {
            return None;
        }
        let entries = (0..=entry)
            .map(|i| Entry {
                event: self.events[i].clone(),
                payload: self.payloads[i].clone(),
            })
            .collect();
        Some(InclusionProof { entries })
    }

    fn export(&self, from: u64, to: u64) -> Option<EvidenceBundle> {
        let from = from as usize;
        let to = to as usize;
        if from > to || to >= self.events.len() {
            return None;
        }
        let entries = (from..=to)
            .map(|i| Entry {
                event: self.events[i].clone(),
                payload: self.payloads[i].clone(),
            })
            .collect();
        Some(EvidenceBundle { entries })
    }
}
