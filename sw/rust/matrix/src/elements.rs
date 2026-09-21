//! What a matrix's elements have in common.

use tagma_core::{Coord, CoordPath};

use crate::order::Order;

/// Reads the elements of a rank-2 matrix.
///
/// Implemented by an owned matrix and by one that borrows a buffer, so a product
/// can read weights that live in flash without copying them into RAM first.
pub trait Elements<const R: usize, const C: usize, O: Order> {
    /// The element at row `i`, column `j`.
    fn element(&self, i: usize, j: usize) -> i8;

    /// The coordinate of the element at row `i`, column `j`.
    ///
    /// `None` when the position lies outside the matrix.
    fn path_of(&self, i: usize, j: usize) -> Option<CoordPath<2>> {
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
/// The bound guards the cast to sixteen bits. The failure arm is unreachable from
/// this crate: every implementor bounds its dimensions by `Coord::N_VALID` at
/// compile time, and callers pass an index below its dimension.
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
