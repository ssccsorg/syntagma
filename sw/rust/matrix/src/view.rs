//! A matrix that borrows its buffer.

use core::marker::PhantomData;

use tagma_core::Coord;

use crate::elements::Elements;
use crate::order::{Order, RowMajor};

/// A rank-2 matrix that borrows its buffer.
///
/// For weights that live in flash. Reading them through this type copies
/// nothing, which an owned matrix cannot promise, so the weights stay where the
/// linker put them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MatrixRef<'a, const R: usize, const C: usize, O: Order = RowMajor> {
    data: &'a [[i8; C]; R],
    order: PhantomData<O>,
}

impl<'a, const R: usize, const C: usize, O: Order> MatrixRef<'a, R, C, O> {
    /// Fails the build when a dimension exceeds the coordinate space.
    const ADDRESSABLE: () = assert!(
        R <= Coord::N_VALID && C <= Coord::N_VALID,
        "tagma-matrix: a dimension exceeds the coordinate space"
    );

    /// Borrows a buffer whose bytes are already in `O` order.
    pub fn new(data: &'a [[i8; C]; R]) -> Self {
        let () = Self::ADDRESSABLE;
        Self {
            data,
            order: PhantomData,
        }
    }

    /// The element at row `i`, column `j`.
    #[inline]
    pub fn get(&self, i: usize, j: usize) -> i8 {
        let offset = O::offset(R, C, i, j);
        self.data[offset / C][offset % C]
    }
}

impl<'a, const R: usize, const C: usize, O: Order> Elements<R, C, O> for MatrixRef<'a, R, C, O> {
    #[inline]
    fn element(&self, i: usize, j: usize) -> i8 {
        self.get(i, j)
    }
}
