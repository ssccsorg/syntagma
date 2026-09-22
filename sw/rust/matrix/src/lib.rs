//! # tagma-matrix: Coordinate-addressed matrices for Tagma
//!
//! Rank-2 `i8` elements addressed by a [`CoordPath`](tagma_core::CoordPath), the
//! integer product over them, the rescale from an accumulator back to an element,
//! and a wire form that moves a matrix to another device.
//!
//! An element's address is its coordinate, so the physical order of the backing
//! bytes is not part of that address, and the same coordinate reads the same value
//! wherever the bytes sit. The product, the rescale of it, and the wire form inherit
//! that: any of them produces the same result for the same logical operands whatever
//! order their storage uses.
//!
//! This crate depends only on [`tagma-core`](tagma_core) and does not modify or
//! replace any existing primitive. It is the family member that takes no
//! allocator, so `tagma-core` is used with `default-features = false`.

#![no_std]

mod elements;
mod gemv;
mod matmul;
mod matrix;
mod order;

#[cfg(kani)]
mod proofs;

mod requant;
mod view;
mod wire;

pub use crate::elements::Elements;
pub use crate::gemv::{gemv, gemv_requantized};
pub use crate::matmul::{matmul, matmul_requantized};
pub use crate::matrix::Matrix;
pub use crate::order::{ColMajor, Order, RowMajor};
pub use crate::requant::{requantize_into, Requant};
pub use crate::view::MatrixRef;
pub use crate::wire::{DecodeError, EncodeError};
