//! A matrix that borrows its buffer.

use core::marker::PhantomData;

use crate::elements::{addressable, Elements};
use crate::order::{Order, RowMajor};

/// A rank-2 matrix that borrows its buffer.
///
/// For weights that live in flash. Reading them through this type copies
/// nothing, which an owned matrix cannot promise, so the weights stay where the
/// linker put them. It writes them out through the same trait method an owned
/// matrix uses, so sending them costs no copy either.
///
/// Both dimensions are bounded by the coordinate space, as they are for an owned
/// matrix, and they are asserted the same way.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MatrixRef<'a, const R: usize, const C: usize, O: Order = RowMajor> {
    data: &'a [[i8; C]; R],
    order: PhantomData<O>,
}

impl<'a, const R: usize, const C: usize, O: Order> MatrixRef<'a, R, C, O> {
    /// Borrows a buffer whose bytes are already in `O` order.
    pub fn new(data: &'a [[i8; C]; R]) -> Self {
        const { addressable(R, C) };
        Self {
            data,
            order: PhantomData,
        }
    }

    /// The element at row `i`, column `j`.
    ///
    /// # Panics
    ///
    /// Panics if `i` is at or above `R` or `j` at or above `C`, for the reason an
    /// owned matrix does: the offset of a position outside the matrix is another
    /// position's offset.
    #[inline]
    pub fn get(&self, i: usize, j: usize) -> i8 {
        assert!(i < R && j < C, "tagma-matrix: position outside the matrix");
        self.read(i, j)
    }

    /// The element at a position already known to be inside the matrix.
    #[inline]
    fn read(&self, i: usize, j: usize) -> i8 {
        let offset = O::offset(R, C, i, j);
        self.data[offset / C][offset % C]
    }
}

impl<'a, const R: usize, const C: usize, O: Order> Elements<R, C, O> for MatrixRef<'a, R, C, O> {
    #[inline]
    fn element(&self, i: usize, j: usize) -> i8 {
        self.read(i, j)
    }
}
