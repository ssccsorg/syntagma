//! tagma-sec: security layer for the synTagma ecosystem.
//!
//! Implements the authority, integrity, audit, and channel primitives
//! specified in `spec/tagma-sec.md`, plus a proxy stack that switches
//! between a legacy security pattern and the tagma-sec pattern against the
//! same scenario-driven workflow tests.

extern crate alloc;

pub mod audit;
pub mod authority;
pub mod channel;
pub mod delos;
pub mod hash;
pub mod integrity;
pub mod legacy;
pub mod proxy;
pub mod types;
