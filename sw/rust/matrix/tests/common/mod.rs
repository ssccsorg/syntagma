//! Fixtures shared by the integration tests.

#![allow(dead_code)]

use tagma_matrix::{ColMajor, Matrix, RowMajor};

/// The logical value at row `i`, column `j`.
///
/// A small signed spread that produces negative products and non-trivial
/// accumulation, so an error in the addressing or in the accumulation changes the
/// output instead of cancelling out.
pub fn value_at(i: usize, j: usize) -> i8 {
    ((i as i32 * 7 + j as i32 * 13) % 61 - 30) as i8
}

/// The logical matrix in row-major order.
pub fn row_major<const R: usize, const C: usize>() -> Matrix<R, C, RowMajor> {
    let mut data = [[0i8; C]; R];
    for (i, row) in data.iter_mut().enumerate() {
        for (j, cell) in row.iter_mut().enumerate() {
            *cell = value_at(i, j);
        }
    }
    Matrix::new(data)
}

/// The same logical matrix in column-major order.
///
/// The buffer is filled by this module's own reading of column-major layout, so
/// the test does not borrow the library's notion of it.
pub fn col_major<const R: usize, const C: usize>() -> Matrix<R, C, ColMajor> {
    let mut data = [[0i8; C]; R];
    for j in 0..C {
        for i in 0..R {
            let offset = j * R + i;
            data[offset / C][offset % C] = value_at(i, j);
        }
    }
    Matrix::new(data)
}

/// A deterministic activation vector.
pub fn activation<const C: usize>() -> [i8; C] {
    let mut x = [0i8; C];
    for (j, cell) in x.iter_mut().enumerate() {
        *cell = ((j as i32 * 11) % 41 - 20) as i8;
    }
    x
}
