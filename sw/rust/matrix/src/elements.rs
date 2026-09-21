//! What a matrix's elements have in common.

use tagma_core::{Coord, CoordPath};

use crate::order::Order;
use crate::wire::EncodeError;

/// Fails the build when a dimension lies outside the coordinate space, or is zero.
///
/// An element's address is a pair of [`Coord`]s, so neither dimension may exceed
/// [`Coord::N_VALID`]: an index above the space has no coordinate to name it. A
/// column count is the divisor when a flat offset is turned back into a position,
/// and a matrix with no row holds no element, so both dimensions are also at
/// least one.
///
/// Every constructor and every derived method below asserts this, so a type whose
/// dimensions break it fails to build rather than panicking later. The assertion
/// is evaluated at codegen, which means a build reports it and `cargo check` on
/// its own does not.
pub(crate) const fn addressable(r: usize, c: usize) {
    assert!(
        r <= Coord::N_VALID && c <= Coord::N_VALID && r > 0 && c > 0,
        "tagma-matrix: a dimension is outside the coordinate space"
    );
}

/// Reads the elements of a rank-2 matrix, and writes it out.
///
/// Implemented by an owned matrix and by one that borrows a buffer, so a product
/// reads weights that live in flash without copying them into RAM first, and a
/// flash-resident matrix is sent on without a copy as well.
///
/// An implementor must bound both dimensions by the coordinate space, because an
/// element's address is a pair of [`Coord`]s: an index at or above
/// [`Coord::N_VALID`] has no coordinate to be named by. Calling any method here,
/// or constructing the shipped types, is what reports a violation.
pub trait Elements<const R: usize, const C: usize, O: Order> {
    /// The element at row `i`, column `j`.
    ///
    /// `i` must be below `R` and `j` below `C`. A position outside the matrix has
    /// no element of its own: its flat offset is another position's, so an
    /// unchecked read answers with a value that belongs to a different coordinate.
    /// The methods below, the shipped product, and the shipped implementors' public
    /// readers stay inside the range.
    fn element(&self, i: usize, j: usize) -> i8;

    /// The coordinate of the element at row `i`, column `j`.
    ///
    /// `None` when the position lies outside the matrix.
    fn path_of(&self, i: usize, j: usize) -> Option<CoordPath<2>> {
        const { addressable(R, C) };
        if i < R && j < C {
            Some(CoordPath::new([coord_of(i), coord_of(j)]))
        } else {
            None
        }
    }

    /// The element a coordinate names.
    ///
    /// `None` when the coordinate names no element of this matrix.
    fn at(&self, path: CoordPath<2>) -> Option<i8> {
        const { addressable(R, C) };
        let coords = path.coords();
        let i = coords[0].index() as usize;
        let j = coords[1].index() as usize;
        if i < R && j < C {
            Some(self.element(i, j))
        } else {
            None
        }
    }

    /// The bytes a stream of this matrix occupies.
    fn encoded_len() -> usize {
        const { addressable(R, C) };
        crate::wire::encoded_len::<R, C>()
    }

    /// Writes the matrix as its coordinates and their values.
    ///
    /// Elements are written in logical order, so two matrices with the same
    /// logical content write the same bytes whatever order their own storage uses.
    /// The operation is here rather than on the owned type because reading is all
    /// it takes, and a matrix whose weights live in flash must not be copied into
    /// RAM to be sent.
    fn encode_into(&self, out: &mut [u8]) -> Result<usize, EncodeError>
    where
        Self: Sized,
    {
        const { addressable(R, C) };
        crate::wire::encode::<R, C, O, Self>(self, out)
    }
}

/// The coordinate of a zero-based index.
///
/// The bound guards the cast to sixteen bits, and it is the runtime backstop for
/// an implementor that declares dimensions outside the coordinate space without
/// calling any method that would report it.
#[inline]
pub(crate) const fn coord_of(index: usize) -> Coord {
    if index < Coord::N_VALID {
        match Coord::new(index as u16) {
            Some(coord) => coord,
            None => panic!("tagma-matrix: index exceeds the coordinate space"),
        }
    } else {
        panic!("tagma-matrix: index exceeds the coordinate space")
    }
}
