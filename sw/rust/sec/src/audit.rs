//! Audit: append-only chained evidence log.
//!
//! `ChainedAudit` is the shared implementation for both patterns: each entry
//! stores the previous entry id, so retroactive modification of an interior
//! entry breaks the chain.

use alloc::vec::Vec;

use crate::types::{Epoch, Event};

pub trait Audit {
    /// Appends an event for `payload` at `epoch` and returns the entry.
    fn append(&mut self, payload: &[u8], epoch: Epoch) -> Event;

    /// Returns true when the chain is intact over entries `from..=to`.
    fn verify_chain(&self, from: u64, to: u64) -> bool;
}

/// Chained append-only log shared by the legacy and tagma-sec stacks.
pub struct ChainedAudit {
    events: Vec<Event>,
}

impl ChainedAudit {
    pub fn new() -> Self {
        Self { events: Vec::new() }
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
}
