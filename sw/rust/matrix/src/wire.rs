//! The wire form of a matrix.
//!
//! A matrix crosses to another device as its coordinates and their values. A
//! receiver places each value at the coordinate it names, so the two sides may
//! keep their bytes in different orders and still agree on every element.
//!
//! A reader refuses a stream it cannot account for: a magic or a version it does
//! not know, a shape that disagrees with the matrix being filled, a length the
//! shape does not account for, or a record whose coordinate names another element
//! than its own position.

use crate::elements::coord_of;
use crate::matrix::Matrix;
use crate::order::Order;

/// The four bytes that open a stream.
const MAGIC: [u8; 4] = *b"TGMX";

/// The encoding this build writes, and the only one it reads.
const VERSION: u8 = 1;

/// Magic, version, rows, and columns.
const HEADER_LEN: usize = 9;

/// Two sixteen-bit coordinates and one value.
const RECORD_LEN: usize = 5;

/// Why a matrix could not be written.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EncodeError {
    /// The destination cannot hold the encoded matrix.
    BufferTooSmall {
        /// The bytes the matrix needs.
        needed: usize,
        /// The bytes the destination offers.
        available: usize,
    },
}

/// Why a stream could not be read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecodeError {
    /// Fewer bytes than a header holds.
    TooShort,
    /// The opening bytes belong to another format.
    BadMagic,
    /// The version is not one this build reads.
    UnsupportedVersion(u8),
    /// The stream's shape is not the shape of the matrix being filled.
    ShapeMismatch {
        /// The rows the stream declares.
        rows: usize,
        /// The columns the stream declares.
        cols: usize,
    },
    /// The stream ends before its records do.
    Truncated,
    /// The stream holds bytes the shape does not account for.
    TrailingBytes {
        /// The bytes past the last record.
        extra: usize,
    },
    /// A record's coordinate names no element of the matrix.
    CoordinateOutOfRange,
    /// A record's coordinate names another element than its own position.
    CoordinateMismatch,
}

impl<const R: usize, const C: usize, O: Order> Matrix<R, C, O> {
    /// The bytes a stream of this matrix occupies.
    pub fn encoded_len() -> usize {
        HEADER_LEN + R * C * RECORD_LEN
    }

    /// Writes the matrix as its coordinates and their values.
    ///
    /// Elements are written in logical order, so two matrices with the same
    /// logical content write the same bytes whatever order their own storage
    /// uses.
    pub fn encode_into(&self, out: &mut [u8]) -> Result<usize, EncodeError> {
        let needed = Self::encoded_len();
        if out.len() < needed {
            return Err(EncodeError::BufferTooSmall {
                needed,
                available: out.len(),
            });
        }

        out[0..4].copy_from_slice(&MAGIC);
        out[4] = VERSION;
        out[5..7].copy_from_slice(&(R as u16).to_le_bytes());
        out[7..9].copy_from_slice(&(C as u16).to_le_bytes());

        let mut cursor = HEADER_LEN;
        for i in 0..R {
            for j in 0..C {
                out[cursor..cursor + 2].copy_from_slice(&coord_of(i).index().to_le_bytes());
                out[cursor + 2..cursor + 4].copy_from_slice(&coord_of(j).index().to_le_bytes());
                out[cursor + 4] = self.get(i, j) as u8;
                cursor += RECORD_LEN;
            }
        }

        Ok(needed)
    }

    /// Reads a matrix from a stream, placing every value at its coordinate.
    pub fn decode(src: &[u8]) -> Result<Self, DecodeError> {
        if src.len() < HEADER_LEN {
            return Err(DecodeError::TooShort);
        }
        if src[0..4] != MAGIC {
            return Err(DecodeError::BadMagic);
        }
        if src[4] != VERSION {
            return Err(DecodeError::UnsupportedVersion(src[4]));
        }

        let rows = u16::from_le_bytes([src[5], src[6]]) as usize;
        let cols = u16::from_le_bytes([src[7], src[8]]) as usize;
        if rows != R || cols != C {
            return Err(DecodeError::ShapeMismatch { rows, cols });
        }

        let needed = Self::encoded_len();
        if src.len() < needed {
            return Err(DecodeError::Truncated);
        }
        if src.len() > needed {
            return Err(DecodeError::TrailingBytes {
                extra: src.len() - needed,
            });
        }

        let mut matrix = Self::new([[0i8; C]; R]);
        let mut cursor = HEADER_LEN;
        for i in 0..R {
            for j in 0..C {
                let row = u16::from_le_bytes([src[cursor], src[cursor + 1]]) as usize;
                let column = u16::from_le_bytes([src[cursor + 2], src[cursor + 3]]) as usize;
                let value = src[cursor + 4] as i8;
                cursor += RECORD_LEN;

                if row >= R || column >= C {
                    return Err(DecodeError::CoordinateOutOfRange);
                }
                if row != i || column != j {
                    return Err(DecodeError::CoordinateMismatch);
                }

                matrix.set(row, column, value);
            }
        }

        Ok(matrix)
    }
}
