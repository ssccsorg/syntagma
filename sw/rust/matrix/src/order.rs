//! The physical order of a matrix's backing bytes.
//!
//! The order decides where a logical element sits. It never decides which
//! element an address names, so two matrices in different orders answer the same
//! coordinate with the same value.

/// Maps a logical element position to its offset in the backing buffer.
pub trait Order {
    /// The offset of the element at row `i`, column `j` in an `R` by `C` matrix.
    fn offset(rows: usize, cols: usize, i: usize, j: usize) -> usize;
}

/// Consecutive columns are adjacent in memory.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RowMajor;

impl Order for RowMajor {
    #[inline]
    fn offset(_rows: usize, cols: usize, i: usize, j: usize) -> usize {
        i * cols + j
    }
}

/// Consecutive rows are adjacent in memory.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColMajor;

impl Order for ColMajor {
    #[inline]
    fn offset(rows: usize, _cols: usize, i: usize, j: usize) -> usize {
        j * rows + i
    }
}
