//! What a matrix's elements have in common.

use tagma_core::{Coord, CoordPath};

use crate::order::Order;

/// Reads the elements of a rank-2 matrix.
///
/// Implemented by an owned matrix and by one that borrows a buffer, so a product
/// can read weights that live in flash without copying them into RAM first.
///
/// An implementor must bound both dimensions by the coordinate space, because an
/// element's address is a pair of [`Coord`]s: an index at or above
/// [`Coord::N_VALID`] has no coordinate to be named by.
pub trait Elements<const R: usize, const C: usize, O: Order> {
    /// Fails the build when a dimension lies outside the coordinate space, or is
    /// zero.
    ///
    /// The bound is stated once, here, and every default method below names this
    /// constant, so an implementor whose dimensions break it fails to compile as
    /// soon as anything addresses or constructs it. Zero is refused because a
    /// column count is the divisor when a flat offset is turned back into a
    /// position, and a matrix with no row holds no element.
    const ADDRESSABLE: () = assert!(
        R <= Coord::N_VALID && C <= Coord::N_VALID && R > 0 && C > 0,
        "tagma-matrix: a dimension is outside the coordinate space"
    );

    /// The element at row `i`, column `j`.
    ///
    /// `i` must be below `R` and `j` below `C`. A position outside the matrix has
    /// no element of its own: its flat offset is another position's, so an
    /// unchecked read answers with a value that belongs to a different coordinate.
    /// The default methods below, the shipped product, and the shipped
    /// implementors' public readers stay inside the range.
    fn element(&self, i: usize, j: usize) -> i8;

    /// The coordinate of the element at row `i`, column `j`.
    ///
    /// `None` when the position lies outside the matrix.
    fn path_of(&self, i: usize, j: usize) -> Option<CoordPath<2>> {
        let () = Self::ADDRESSABLE;
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
        let () = Self::ADDRESSABLE;
        let coords = path.coords();
        let i = coords[0].index() as usize;
        let j = coords[1].index() as usize;
        if i < R && j < C {
            Some(self.element(i, j))
        } else {
            None
        }
    }
}

/// The coordinate of a zero-based index.
///
/// The bound guards the cast to sixteen bits, and it is the runtime backstop for
/// an implementor that declares dimensions outside the coordinate space without
/// addressing them, so the constant above never fires.
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
