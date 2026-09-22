//! Fixtures shared by the integration tests.

#![allow(dead_code)]

use tagma_matrix::{ColMajor, Elements, Matrix, Order, Requant, RowMajor};

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

/// The product of two matrices, written through the coordinate surface.
///
/// Both operands are reached through their coordinates rather than through the
/// order-resolved index, so the shipped contraction and this one do not share an
/// access path.
pub fn matmul_reference<
    const R: usize,
    const K: usize,
    const C: usize,
    OA: Order,
    OB: Order,
    A: Elements<R, K, OA>,
    B: Elements<K, C, OB>,
>(
    a: &A,
    b: &B,
) -> [[i32; C]; R] {
    let mut out = [[0i32; C]; R];
    for (i, row) in out.iter_mut().enumerate() {
        for (j, element) in row.iter_mut().enumerate() {
            let mut accumulator: i32 = 0;
            for k in 0..K {
                let left = a
                    .at(a.path_of(i, k).expect("reference: the row is in range"))
                    .expect("reference: the coordinate names an element");
                let right = b
                    .at(b.path_of(k, j).expect("reference: the row is in range"))
                    .expect("reference: the coordinate names an element");
                accumulator += i32::from(left) * i32::from(right);
            }
            *element = accumulator;
        }
    }
    out
}

/// The identity of a size, which leaves the other operand of a product unchanged.
pub fn identity<const N: usize>() -> Matrix<N, N, RowMajor> {
    let mut data = [[0i8; N]; N];
    for (i, row) in data.iter_mut().enumerate() {
        row[i] = 1;
    }
    Matrix::new(data)
}

/// The rescale the tests use.
///
/// A value a little below one, carried the way a quantized model carries it: a
/// multiplier that fills a signed thirty-two bit integer and a shift that makes it a
/// fraction.
pub fn requant() -> Requant {
    Requant::new(38_997_123, 27)
}

/// A rescale a little above one, so a run covers both sides of the fixed point.
pub fn requant_above_one() -> Requant {
    Requant::new(2_147_483, 20)
}

/// The rescale of one accumulator, written as a floor division.
///
/// The crate reaches the result through an arithmetic shift, which floors, and this
/// reference reaches it through the language's division, which truncates, with the
/// floor applied by hand. The two agree only if the rounding is the one the crate
/// documents, so this checks the direction of the rounding rather than only the width
/// of the intermediate.
pub fn requant_reference(accumulator: i32, rescale: &Requant) -> i8 {
    let product = i128::from(accumulator) * i128::from(rescale.multiplier());
    let divisor = 1i128 << rescale.shift();
    let rounding = divisor / 2;
    let shifted = floor_divide(product + rounding, divisor);

    if shifted > i128::from(i8::MAX) {
        i8::MAX
    } else if shifted < i128::from(i8::MIN) {
        i8::MIN
    } else {
        shifted as i8
    }
}

/// Division that floors, which the language's operator does not do for a negative
/// numerator.
fn floor_divide(numerator: i128, denominator: i128) -> i128 {
    let quotient = numerator / denominator;
    if numerator % denominator != 0 && (numerator < 0) != (denominator < 0) {
        quotient - 1
    } else {
        quotient
    }
}
