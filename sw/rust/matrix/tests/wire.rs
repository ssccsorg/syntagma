//! The reader's refusals.
//!
//! A stream this format cannot account for is refused with the reason, rather
//! than read as something plausible.

mod common;

use tagma_matrix::{DecodeError, EncodeError, Matrix, RowMajor};

const R: usize = 4;
const C: usize = 3;
const WIRE: usize = 9 + R * C * 5;

type Target = Matrix<R, C, RowMajor>;

/// A stream this format wrote, from the shared fixture.
fn stream() -> [u8; WIRE] {
    let matrix = common::row_major::<R, C>();
    let mut bytes = [0u8; WIRE];
    matrix
        .encode_into(&mut bytes)
        .expect("the buffer holds the matrix");
    bytes
}

/// The offset of the first byte of the record at position `k`.
fn record(k: usize) -> usize {
    9 + k * 5
}

#[test]
fn a_valid_stream_reads_back() {
    let matrix = Target::decode(&stream()).expect("a stream this format wrote");

    for i in 0..R {
        for j in 0..C {
            assert_eq!(matrix.get(i, j), common::value_at(i, j));
        }
    }
}

#[test]
fn a_short_header_is_refused() {
    assert_eq!(
        Target::decode(&stream()[..8]).unwrap_err(),
        DecodeError::TooShort
    );
}

#[test]
fn a_foreign_magic_is_refused() {
    let mut bytes = stream();
    bytes[0] = b'X';
    assert_eq!(Target::decode(&bytes).unwrap_err(), DecodeError::BadMagic);
}

#[test]
fn an_unknown_version_is_refused() {
    let mut bytes = stream();
    bytes[4] = 9;
    assert_eq!(
        Target::decode(&bytes).unwrap_err(),
        DecodeError::UnsupportedVersion(9)
    );
}

#[test]
fn a_shape_that_disagrees_is_refused() {
    let mut bytes = stream();
    bytes[5..7].copy_from_slice(&(R as u16 + 1).to_le_bytes());
    assert_eq!(
        Target::decode(&bytes).unwrap_err(),
        DecodeError::ShapeMismatch {
            rows: R + 1,
            cols: C,
        }
    );
}

#[test]
fn a_record_section_that_ends_early_is_refused() {
    assert_eq!(
        Target::decode(&stream()[..WIRE - 1]).unwrap_err(),
        DecodeError::Truncated
    );
}

#[test]
fn trailing_bytes_are_refused() {
    let mut bytes = [0u8; WIRE + 1];
    bytes[..WIRE].copy_from_slice(&stream());
    assert_eq!(
        Target::decode(&bytes).unwrap_err(),
        DecodeError::TrailingBytes { extra: 1 }
    );
}

#[test]
fn a_coordinate_outside_the_matrix_is_refused() {
    let mut bytes = stream();
    let last = record(R * C - 1);
    bytes[last..last + 2].copy_from_slice(&(R as u16 + 1).to_le_bytes());
    assert_eq!(
        Target::decode(&bytes).unwrap_err(),
        DecodeError::CoordinateOutOfRange
    );
}

#[test]
fn a_record_in_another_position_is_refused() {
    let mut bytes = stream();
    for offset in 0..5 {
        bytes.swap(record(0) + offset, record(1) + offset);
    }
    assert_eq!(
        Target::decode(&bytes).unwrap_err(),
        DecodeError::CoordinateMismatch
    );
}

#[test]
fn a_destination_that_is_too_small_is_refused() {
    let matrix = common::row_major::<R, C>();
    let mut small = [0u8; WIRE - 1];
    assert_eq!(
        matrix.encode_into(&mut small).unwrap_err(),
        EncodeError::BufferTooSmall {
            needed: WIRE,
            available: WIRE - 1,
        }
    );
}
