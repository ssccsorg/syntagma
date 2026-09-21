//! An owned matrix.

use core::marker::PhantomData;

use tagma_core::Coord;

use crate::elements::Elements;
use crate::order::{Order, RowMajor};

/// A rank-2 matrix that owns its buffer.
///
/// `R` is the row count, `C` the column count, and `O` the physical order of the
/// backing bytes. The order decides where an element sits and never which element
/// an address names, so two matrices in different orders answer the same
/// coordinate with the same value.
///
/// Both dimensions are bounded by the coordinate space. A matrix larger than the
/// space fails to compile rather than truncating an address.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Matrix<const R: usize, const C: usize, O: Order = RowMajor> {
    data: [[i8; C]; R],
    order: PhantomData<O>,
}

impl<const R: usize, const C: usize, O: Order> Matrix<R, C, O> {
    /// Fails the build when a dimension exceeds the coordinate space.
    const ADDRESSABLE: () = assert!(
        R <= Coord::N_VALID && C <= Coord::N_VALID,
        "tagma-matrix: a dimension exceeds the coordinate space"
    );

    /// Wraps a buffer whose bytes are already in `O` order.
    pub fn new(data: [[i8; C]; R]) -> Self {
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

    /// Sets the element at row `i`, column `j`.
    #[inline]
    pub fn set(&mut self, i: usize, j: usize, value: i8) {
        let offset = O::offset(R, C, i, j);
        self.data[offset / C][offset % C] = value;
    }
}

impl<const R: usize, const C: usize, O: Order> Elements<R, C, O> for Matrix<R, C, O> {
    #[inline]
    fn element(&self, i: usize, j: usize) -> i8 {
        self.get(i, j)
    }
}
