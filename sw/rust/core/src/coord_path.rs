use crate::Coord;

/// An index path through a `CoordSpace`, not a key.
///
/// Each element selects one of 11,172 slots at the corresponding tree depth.
/// `N` is a compile-time depth tag ensuring the path length matches the map.
///
/// # Index path vs key
///
/// `CoordPath` is **not** a hash map key. It is a path specifier:
/// each coordinate is used directly as an array index at one tree level.
/// No hashing, no equality comparison — the coordinate *is* the address.
///
/// # Examples
///
/// ```
/// use tagma_core::{Coord, CoordPath};
///
/// let ga = Coord::new(0).unwrap();
/// let path = CoordPath::<1>::new([ga]);
/// assert_eq!(path.coords()[0], ga);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct CoordPath<const N: usize> {
    coords: [Coord; N],
}

impl<const N: usize> CoordPath<N> {
    /// Creates a new `CoordPath` from an array of coordinates.
    ///
    /// No validity check beyond what `Coord` already guarantees:
    /// all coords are structurally valid by construction.
    #[inline]
    pub const fn new(coords: [Coord; N]) -> Self {
        CoordPath { coords }
    }

    /// Returns a reference to the internal coordinate array.
    #[inline]
    pub const fn coords(&self) -> &[Coord; N] {
        &self.coords
    }

    /// Returns the coordinate at the given index in the path.
    #[inline]
    pub fn get(&self, index: usize) -> Option<&Coord> {
        self.coords.get(index)
    }

    /// Returns the number of coordinates in the path (always `N`).
    #[inline]
    pub const fn len(&self) -> usize {
        N
    }

    /// Returns `true` if the path is empty (`N == 0`).
    ///
    /// Always `false` for `CoordPath` since `N >= 1` in practice.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        N == 0
    }

    /// Iterates over the coordinates in the path.
    #[inline]
    pub fn iter(&self) -> core::slice::Iter<'_, Coord> {
        self.coords.iter()
    }
}

impl<const N: usize> PartialEq for CoordPath<N> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.coords == other.coords
    }
}

impl<const N: usize> Eq for CoordPath<N> {}

impl<const N: usize> core::fmt::Display for CoordPath<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CoordPath<{}>(", N)?;
        for (i, coord) in self.coords.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", coord)?;
        }
        write!(f, ")")
    }
}

// ── Conversions ──────────────────────────────────────────

/// A single-coordinate path (`CoordPath<1>`) — the most common case.
impl From<Coord> for CoordPath<1> {
    #[inline]
    fn from(coord: Coord) -> Self {
        CoordPath::new([coord])
    }
}

impl From<&Coord> for CoordPath<1> {
    #[inline]
    fn from(coord: &Coord) -> Self {
        CoordPath::new([*coord])
    }
}

impl<const N: usize> From<[Coord; N]> for CoordPath<N> {
    #[inline]
    fn from(coords: [Coord; N]) -> Self {
        CoordPath::new(coords)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Coord;

    #[test]
    fn path_from_coord() {
        let c = Coord::new(42).unwrap();
        let path = CoordPath::<1>::new([c]);
        assert_eq!(path.coords()[0], c);
        assert_eq!(path.len(), 1);
    }

    #[test]
    fn path_from_array() {
        let a = Coord::new(0).unwrap();
        let b = Coord::new(1).unwrap();
        let path = CoordPath::<2>::new([a, b]);
        assert_eq!(path.coords()[0], a);
        assert_eq!(path.coords()[1], b);
    }

    #[test]
    fn path_into_conversion() {
        let c = Coord::new(7).unwrap();
        let path: CoordPath<1> = c.into();
        assert_eq!(path.coords()[0], c);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn path_display() {
        let c = Coord::new(0).unwrap();
        let path = CoordPath::<1>::new([c]);
        let s = alloc::format!("{}", path);
        assert!(s.contains("가"));
    }

    #[test]
    fn path_eq() {
        let a = CoordPath::<2>::new([Coord::new(0).unwrap(), Coord::new(1).unwrap()]);
        let b = CoordPath::<2>::new([Coord::new(0).unwrap(), Coord::new(1).unwrap()]);
        assert_eq!(a, b);
    }

    #[test]
    fn path_ne() {
        let a = CoordPath::<2>::new([Coord::new(0).unwrap(), Coord::new(1).unwrap()]);
        let b = CoordPath::<2>::new([Coord::new(0).unwrap(), Coord::new(2).unwrap()]);
        assert_ne!(a, b);
    }
}
