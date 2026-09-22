//! Relocation invariance.
//!
//! The same logical matrix in two physical orders must answer the same
//! coordinates with the same values, produce the same product, and write the
//! same bytes. This is what makes the teleport claim falsifiable: if an address
//! depended on where the bytes sit, these runs would differ.

mod common;

use tagma_matrix::{gemv, gemv_requantized, ColMajor, Elements, Matrix};

const R: usize = 9;
const C: usize = 6;
const WIRE: usize = 9 + R * C * 5;

#[test]
fn two_physical_orders_answer_the_same_coordinates() {
    let rows = common::row_major::<R, C>();
    let columns = common::col_major::<R, C>();

    for i in 0..R {
        for j in 0..C {
            assert_eq!(rows.path_of(i, j), columns.path_of(i, j));
            assert_eq!(
                rows.at(rows.path_of(i, j).expect("in range")),
                Some(common::value_at(i, j))
            );
            assert_eq!(
                columns.at(columns.path_of(i, j).expect("in range")),
                Some(common::value_at(i, j))
            );
        }
    }
}

#[test]
fn two_physical_orders_produce_a_byte_identical_product() {
    let rows = common::row_major::<R, C>();
    let columns = common::col_major::<R, C>();
    let x = common::activation::<C>();

    let mut from_rows = [0i32; R];
    let mut from_columns = [0i32; R];
    gemv(&rows, &x, &mut from_rows);
    gemv(&columns, &x, &mut from_columns);

    assert_eq!(from_rows, from_columns);
}

/// The rescale is what a model's next layer consumes, so the invariance has to hold
/// of the element rather than only of the accumulator it came from.
#[test]
fn two_physical_orders_produce_a_byte_identical_requantized_product() {
    let rows = common::row_major::<R, C>();
    let columns = common::col_major::<R, C>();
    let x = common::activation::<C>();
    let rescale = common::requant();

    let mut from_rows = [0i8; R];
    let mut from_columns = [0i8; R];
    gemv_requantized(&rows, &x, &rescale, &mut from_rows);
    gemv_requantized(&columns, &x, &rescale, &mut from_columns);

    assert_eq!(from_rows, from_columns);
}

#[test]
fn the_requantized_product_is_the_rescale_of_the_product() {
    let rows = common::row_major::<R, C>();
    let x = common::activation::<C>();
    let rescale = common::requant();

    let mut accumulators = [0i32; R];
    gemv(&rows, &x, &mut accumulators);

    let mut expected = [0i8; R];
    for (accumulator, element) in accumulators.iter().zip(expected.iter_mut()) {
        *element = rescale.requantize(*accumulator);
    }

    let mut from_rows = [0i8; R];
    gemv_requantized(&rows, &x, &rescale, &mut from_rows);

    assert_eq!(from_rows, expected);
}

#[test]
fn a_round_trip_carries_the_matrix_into_another_order() {
    let rows = common::row_major::<R, C>();
    let x = common::activation::<C>();

    let mut bytes = [0u8; WIRE];
    let written = rows
        .encode_into(&mut bytes)
        .expect("the buffer holds the matrix");
    assert_eq!(written, WIRE);

    let columns = Matrix::<R, C, ColMajor>::decode(&bytes).expect("a stream this format wrote");

    for i in 0..R {
        for j in 0..C {
            assert_eq!(
                columns.at(columns.path_of(i, j).expect("in range")),
                Some(common::value_at(i, j))
            );
        }
    }

    let mut from_rows = [0i32; R];
    let mut from_columns = [0i32; R];
    gemv(&rows, &x, &mut from_rows);
    gemv(&columns, &x, &mut from_columns);
    assert_eq!(from_rows, from_columns);
}

#[test]
fn the_stream_does_not_depend_on_the_physical_order() {
    let rows = common::row_major::<R, C>();
    let columns = common::col_major::<R, C>();

    let mut from_rows = [0u8; WIRE];
    let mut from_columns = [0u8; WIRE];
    rows.encode_into(&mut from_rows)
        .expect("the buffer holds the matrix");
    columns
        .encode_into(&mut from_columns)
        .expect("the buffer holds the matrix");

    assert_eq!(from_rows, from_columns);
}

#[test]
fn a_relocated_value_writes_the_same_bytes_it_arrived_in() {
    let rows = common::row_major::<R, C>();

    let mut sent = [0u8; WIRE];
    rows.encode_into(&mut sent)
        .expect("the buffer holds the matrix");

    let received = Matrix::<R, C, ColMajor>::decode(&sent).expect("a stream this format wrote");

    let mut resent = [0u8; WIRE];
    received
        .encode_into(&mut resent)
        .expect("the buffer holds the matrix");

    assert_eq!(sent, resent);
}
