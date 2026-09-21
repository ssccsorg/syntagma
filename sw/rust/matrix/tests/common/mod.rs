//! Fixtures shared by the integration tests.

#![allow(dead_code)]

use tagma_matrix::{ColMajor, Elements, Matrix, Order, RowMajor};

/// The logical value at row `i`, column `j`.
///
/// A small signed spread that produces negative products and non-trivial
/// accumulation, so an error in the addressing or in the accumulation changes the
/// output instead of cancelling out.
pub fn value_at(i: usize, j: usize) -> i8 {
    ((i as i32 * 7 + j as i32 * 13) % 61 - 30) as i8
}

/// The logical values laid out row-major.
pub fn row_major_buffer<const R: usize, const C: usize>() -> [[i8; C]; R] {
    let mut data = [[0i8; C]; R];
    for (i, row) in data.iter_mut().enumerate() {
        for (j, cell) in row.iter_mut().enumerate() {
            *cell = value_at(i, j);
        }
    }
    data
}

/// The logical matrix in row-major order.
pub fn row_major<const R: usize, const C: usize>() -> Matrix<R, C, RowMajor> {
    Matrix::new(row_major_buffer::<R, C>())
}

/// The same logical values laid out column-major.
///
/// The buffer is filled by this module's own reading of column-major layout, so
/// the test does not borrow the library's notion of it.
pub fn col_major_buffer<const R: usize, const C: usize>() -> [[i8; C]; R] {
    let mut data = [[0i8; C]; R];
    for j in 0..C {
        for i in 0..R {
            let offset = j * R + i;
            data[offset / C][offset % C] = value_at(i, j);
        }
    }
    data
}

/// The same logical matrix in column-major order.
pub fn col_major<const R: usize, const C: usize>() -> Matrix<R, C, ColMajor> {
    Matrix::new(col_major_buffer::<R, C>())
}

/// A deterministic activation vector.
pub fn activation<const C: usize>() -> [i8; C] {
    let mut x = [0i8; C];
    for (j, cell) in x.iter_mut().enumerate() {
        *cell = ((j as i32 * 11) % 41 - 20) as i8;
    }
    x
}

/// The product written as the plainest possible loop, reading every element
/// through its coordinate rather than through the order-resolved index, so the
/// two access paths the crate offers are checked against a third.
pub fn reference<const R: usize, const C: usize, O: Order, E: Elements<R, C, O>>(
    a: &E,
    x: &[i8; C],
) -> [i32; R] {
    let mut y = [0i32; R];
    for (i, out) in y.iter_mut().enumerate() {
        let mut accumulator: i32 = 0;
        for (j, activation) in x.iter().enumerate() {
            let path = match a.path_of(i, j) {
                Some(path) => path,
                None => panic!("reference: position {i}, {j} is outside the matrix"),
            };
            let weight = match a.at(path) {
                Some(weight) => weight,
                None => panic!("reference: coordinate {path} names no element"),
            };
            accumulator += i32::from(weight) * i32::from(*activation);
        }
        *out = accumulator;
    }
    y
}
