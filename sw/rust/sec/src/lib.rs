//! Crate documentation is generated from the README, whose code blocks are
//! compiled as doctests.
#![doc = include_str!("../README.md")]

extern crate alloc;

pub mod audit;
pub mod authority;
pub mod channel;
pub mod delos;
pub(crate) mod hash;
pub mod integrity;
pub mod legacy;
pub mod proxy;
pub mod types;
