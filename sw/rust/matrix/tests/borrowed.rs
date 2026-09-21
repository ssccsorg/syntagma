//! The borrowed matrix.
//!
//! This is the path weights take when they live in flash: the buffer is not copied
//! into RAM, so every read has to reach the same bytes through the same address
//! arithmetic an owned matrix uses. The tests therefore check the borrowed type
//! against the owned one, against the scalar reference, and across the two
//! physical orders.

mod common;

use tagma_matrix::{gemv, ColMajor, Elements, MatrixRef, RowMajor};

const R: usize = 9;
const C: usize = 6;

#[test]
fn a_borrowed_matrix_answers_what_the_owned_one_answers() {
    let owned = common::row_major::<R, C>();
    let buffer = common::row_major_buffer::<R, C>();
    let borrowed = MatrixRef::<R, C, RowMajor>::new(&buffer);

    for i in 0..R {
        for j in 0..C {
            let expected = common::value_at(i, j);
            assert_eq!(borrowed.get(i, j), expected);
            assert_eq!(borrowed.get(i, j), owned.get(i, j));
            assert_eq!(borrowed.path_of(i, j), owned.path_of(i, j));
            assert_eq!(
                borrowed.at(borrowed.path_of(i, j).expect("in range")),
                Some(expected)
            );
        }
    }
}

#[test]
fn a_borrowed_matrix_produces_what_the_owned_one_produces() {
    let owned = common::row_major::<R, C>();
    let buffer = common::row_major_buffer::<R, C>();
    let borrowed = MatrixRef::<R, C, RowMajor>::new(&buffer);
    let x = common::activation::<C>();

    let mut from_owned = [0i32; R];
    let mut from_borrowed = [0i32; R];
    gemv(&owned, &x, &mut from_owned);
    gemv(&borrowed, &x, &mut from_borrowed);

    assert_eq!(from_borrowed, from_owned);
    assert_eq!(from_borrowed, common::reference(&borrowed, &x));
}

#[test]
fn a_borrowed_matrix_is_relocation_invariant() {
    let rows_buffer = common::row_major_buffer::<R, C>();
    let columns_buffer = common::col_major_buffer::<R, C>();
    let rows = MatrixRef::<R, C, RowMajor>::new(&rows_buffer);
    let columns = MatrixRef::<R, C, ColMajor>::new(&columns_buffer);
    let x = common::activation::<C>();

    let mut from_rows = [0i32; R];
    let mut from_columns = [0i32; R];
    gemv(&rows, &x, &mut from_rows);
    gemv(&columns, &x, &mut from_columns);

    assert_eq!(from_rows, from_columns);
    for i in 0..R {
        for j in 0..C {
            assert_eq!(rows.path_of(i, j), columns.path_of(i, j));
        }
    }
}

#[test]
fn a_position_outside_a_borrowed_matrix_has_no_coordinate() {
    let buffer = common::row_major_buffer::<R, C>();
    let borrowed = MatrixRef::<R, C, RowMajor>::new(&buffer);

    assert_eq!(borrowed.path_of(R, 0), None);
    assert_eq!(borrowed.path_of(0, C), None);
}
