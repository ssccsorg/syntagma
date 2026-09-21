//! The address surface: what a position outside the matrix does.
//!
//! A position outside the matrix has no element of its own, and its flat offset
//! equals the offset of a position that is inside it. A reader that answered with
//! that other element would return a value belonging to a different coordinate,
//! which contradicts the one thing this crate promises, so the readers refuse the
//! position instead of resolving it.

mod common;

use tagma_matrix::{MatrixRef, RowMajor};

/// The position whose offset equals the offset of row `1`, column `0` in
/// row-major order, which is what an unchecked read used to return.
#[test]
#[should_panic(expected = "position outside the matrix")]
fn a_column_one_past_the_last_refuses_to_be_read() {
    let a = common::row_major::<2, 3>();
    let _ = a.get(0, 3);
}

#[test]
#[should_panic(expected = "position outside the matrix")]
fn a_row_one_past_the_last_refuses_to_be_read() {
    let a = common::row_major::<2, 3>();
    let _ = a.get(2, 0);
}

#[test]
#[should_panic(expected = "position outside the matrix")]
fn a_column_one_past_the_last_refuses_to_be_read_in_column_major_order() {
    let a = common::col_major::<2, 3>();
    let _ = a.get(0, 3);
}

#[test]
#[should_panic(expected = "position outside the matrix")]
fn a_position_outside_the_matrix_refuses_to_be_written() {
    let mut a = common::row_major::<2, 3>();
    a.set(2, 0, 7);
}

#[test]
#[should_panic(expected = "position outside the matrix")]
fn a_borrowed_read_outside_the_matrix_is_refused() {
    let buffer = common::row_major_buffer::<2, 3>();
    let a = MatrixRef::<2, 3, RowMajor>::new(&buffer);
    let _ = a.get(0, 3);
}
