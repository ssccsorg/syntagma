//! An owned matrix.

use core::marker::PhantomData;

use crate::elements::{addressable, Elements};
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
///
/// A matrix is `Copy`, as the family's other fixed-size types are. A copy
/// duplicates the whole buffer, so a large matrix is a value to borrow or to read
/// through [`MatrixRef`](crate::MatrixRef) rather than to pass around by value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Matrix<const R: usize, const C: usize, O: Order = RowMajor> {
    data: [[i8; C]; R],
    order: PhantomData<O>,
}

impl<const R: usize, const C: usize, O: Order> Matrix<R, C, O> {
    /// Wraps a buffer whose bytes are already in `O` order.
    pub fn new(data: [[i8; C]; R]) -> Self {
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
    /// Panics if `i` is at or above `R` or `j` at or above `C`. The flat offset of
    /// a position outside the matrix is another position's offset, so reading it
    /// unchecked answers with a value that belongs to a different coordinate.
    #[inline]
    pub fn get(&self, i: usize, j: usize) -> i8 {
        assert!(i < R && j < C, "tagma-matrix: position outside the matrix");
        self.read(i, j)
    }

    /// Sets the element at row `i`, column `j`.
    ///
    /// # Panics
    ///
    /// Panics if `i` is at or above `R` or `j` at or above `C`.
    #[inline]
    pub fn set(&mut self, i: usize, j: usize, value: i8) {
        assert!(i < R && j < C, "tagma-matrix: position outside the matrix");
        let offset = O::offset(R, C, i, j);
        self.data[offset / C][offset % C] = value;
    }

    /// The element at a position already known to be inside the matrix.
    ///
    /// The coordinate surface and the product call this, having established the
    /// range themselves.
    #[inline]
    fn read(&self, i: usize, j: usize) -> i8 {
        let offset = O::offset(R, C, i, j);
        self.data[offset / C][offset % C]
    }
}

impl<const R: usize, const C: usize, O: Order> Elements<R, C, O> for Matrix<R, C, O> {
    #[inline]
    fn element(&self, i: usize, j: usize) -> i8 {
        self.read(i, j)
    }
}
