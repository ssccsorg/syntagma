//! The product of two matrices.
//!
//! A contraction is the operation the crate exists for, so it is checked against a
//! reference that reads both operands through their coordinates, against the
//! identity, and across the physical orders of both operands and of the destination.

mod common;

use tagma_matrix::{matmul, matmul_requantized, requantize_into, ColMajor, Matrix, RowMajor};

#[test]
fn the_product_matches_the_coordinate_reference() {
    const R: usize = 4;
    const K: usize = 5;
    const C: usize = 3;

    let a = common::row_major::<R, K>();
    let b = common::row_major::<K, C>();
    let mut out = [[0i32; C]; R];

    matmul(&a, &b, &mut out);

    assert_eq!(out, common::matmul_reference(&a, &b));
}

#[test]
fn an_identity_operand_leaves_the_other_unchanged() {
    const R: usize = 4;
    const C: usize = 3;

    let identity = common::identity::<R>();
    let b = common::row_major::<R, C>();
    let mut out = [[0i32; C]; R];

    matmul(&identity, &b, &mut out);

    for (i, row) in out.iter().enumerate() {
        for (j, value) in row.iter().enumerate() {
            assert_eq!(*value, i32::from(b.get(i, j)));
        }
    }
}

#[test]
fn two_physical_orders_produce_a_byte_identical_product() {
    const R: usize = 4;
    const K: usize = 5;
    const C: usize = 3;

    let rows = common::row_major::<R, K>();
    let columns = common::col_major::<R, K>();

    let left = common::row_major::<K, C>();
    let right = common::col_major::<K, C>();

    let mut from_rows = [[0i32; C]; R];
    let mut from_columns = [[0i32; C]; R];
    matmul(&rows, &left, &mut from_rows);
    matmul(&columns, &right, &mut from_columns);

    assert_eq!(from_rows, from_columns);
}

/// A contraction one term long, which is the smallest shape the sum can take and the
/// one where an off-by-one in the loop bound shows up as a wrong value rather than as
/// a missing one.
#[test]
fn an_outer_product_contracts_over_one_term() {
    const R: usize = 3;
    const C: usize = 4;

    let a = common::row_major::<R, 1>();
    let b = common::row_major::<1, C>();
    let mut out = [[0i32; C]; R];

    matmul(&a, &b, &mut out);

    let expected = common::matmul_reference(&a, &b);
    for (row, expected_row) in out.iter().zip(expected.iter()) {
        assert_eq!(row, expected_row);
    }
}

#[test]
fn the_requantized_product_is_the_rescale_of_the_product() {
    const R: usize = 4;
    const K: usize = 5;
    const C: usize = 3;

    let a = common::row_major::<R, K>();
    let b = common::col_major::<K, C>();
    let rescale = common::requant();

    let mut accumulators = [[0i32; C]; R];
    matmul(&a, &b, &mut accumulators);

    let mut expected = [[0i8; C]; R];
    for (row, elements) in accumulators.iter().zip(expected.iter_mut()) {
        requantize_into(row, &rescale, elements);
    }

    let mut out = Matrix::<R, C, RowMajor>::new([[0i8; C]; R]);
    matmul_requantized(&a, &b, &rescale, &mut out);

    for (i, expected_row) in expected.iter().enumerate() {
        for (j, value) in expected_row.iter().enumerate() {
            assert_eq!(out.get(i, j), *value);
        }
    }
}

#[test]
fn the_requantized_product_does_not_depend_on_the_destination_order() {
    const R: usize = 4;
    const K: usize = 5;
    const C: usize = 3;

    let a = common::row_major::<R, K>();
    let b = common::row_major::<K, C>();
    let rescale = common::requant();

    let mut rows = Matrix::<R, C, RowMajor>::new([[0i8; C]; R]);
    let mut columns = Matrix::<R, C, ColMajor>::new([[0i8; C]; R]);
    matmul_requantized(&a, &b, &rescale, &mut rows);
    matmul_requantized(&a, &b, &rescale, &mut columns);

    for i in 0..R {
        for j in 0..C {
            assert_eq!(rows.get(i, j), columns.get(i, j));
        }
    }
}
