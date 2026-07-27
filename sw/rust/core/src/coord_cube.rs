use crate::coord::Coord;
use crate::coord_path::CoordPath;

/// An interpretation layer over [`CoordPath`] that views `N` characters
/// as a D-dimensional grid where each dimension has `R` characters of
/// resolution (i.e., 11,172^R addressable values per dimension).
///
/// # Const generic constraint
///
/// `N` must equal `D * R`.  This is enforced at runtime via
/// [`CoordCube::from_path`].
///
/// # Tradeoff: Precision vs. Spatial Power
///
/// | Approach | Precision | Spatial ability |
/// |----------|-----------|-----------------|
/// | `CoordPath` (1D) | Highest (per-character axes) | None |
/// | `CoordCube` (D-dim) | Lower (grouped axes) | Rich (distance, region, proximity) |
///
/// The total addressable space is exactly `11,172^N` in both views.
/// `CoordCube` never modifies or replaces `CoordPath` — it is an optional
/// interpretation layer over the same bytes.
///
/// # What CoordCube is not
///
/// - `CoordCube` is **not** a storage key.  Storage always uses `CoordPath`.
/// - `CoordCube` does **not** compute distance metrics.  Those belong in
///   `tagma-geo` or higher-level crates.
///
/// # Type Parameters
///
/// * `N` — Total number of characters.
/// * `D` — Number of spatial dimensions.
/// * `R` — Number of characters per dimension (resolution exponent).
///
/// # Example
///
/// ```
/// use tagma_core::{Coord, CoordPath, CoordCube};
///
/// // N = 6, D = 3, R = 2: 3 dimensions, 2 characters each
/// let path = CoordPath::<6>::new([
///     Coord::new(0).unwrap(),   // dim 0, character 0
///     Coord::new(1).unwrap(),   // dim 0, character 1
///     Coord::new(2).unwrap(),   // dim 1, character 0
///     Coord::new(3).unwrap(),   // dim 1, character 1
///     Coord::new(4).unwrap(),   // dim 2, character 0
///     Coord::new(5).unwrap(),   // dim 2, character 1
/// ]);
///
/// let cube = CoordCube::<6, 3, 2>::from_path(path);
///
/// let axis0 = cube.axis(0);
/// assert_eq!(axis0.coords()[0].index(), 0);
/// assert_eq!(axis0.coords()[1].index(), 1);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct CoordCube<const N: usize, const D: usize, const R: usize> {
    path: CoordPath<N>,
}

// ---------------------------------------------------------------------------
// Construction / Conversion
// ---------------------------------------------------------------------------

impl<const N: usize, const D: usize, const R: usize> CoordCube<N, D, R> {
    /// Creates a `CoordCube` from a `CoordPath`.
    ///
    /// # Panics
    ///
    /// Panics if `N != D * R` (must hold by construction).
    #[inline]
    pub fn from_path(path: CoordPath<N>) -> Self {
        assert!(
            D * R == N,
            "CoordCube: N={} must equal D*R = {}*{} = {}",
            N,
            D,
            R,
            D * R
        );
        CoordCube { path }
    }

    /// Creates a `CoordCube` from a `CoordPath` without checking `N == D * R`.
    ///
    /// # Safety
    ///
    /// Caller must ensure `D * R == N`.
    #[inline]
    pub unsafe fn from_path_unchecked(path: CoordPath<N>) -> Self {
        CoordCube { path }
    }

    /// Returns a reference to the underlying `CoordPath`.
    #[inline]
    pub const fn as_path(&self) -> &CoordPath<N> {
        &self.path
    }

    /// Consumes the cube and returns the underlying `CoordPath`.
    #[inline]
    pub fn into_path(self) -> CoordPath<N> {
        self.path
    }
}

// ---------------------------------------------------------------------------
// Accessors
// ---------------------------------------------------------------------------

impl<const N: usize, const D: usize, const R: usize> CoordCube<N, D, R> {
    /// Returns the number of spatial dimensions.
    #[inline]
    pub const fn ndim(&self) -> usize {
        D
    }

    /// Returns the number of characters per dimension.
    #[inline]
    pub const fn resolution(&self) -> usize {
        R
    }

    /// Returns the total number of characters.
    #[inline]
    pub const fn total_characters(&self) -> usize {
        N
    }

    /// Returns the `R`-character path for dimension `dim`.
    ///
    /// # Panics
    ///
    /// Panics if `dim >= D`.
    pub fn axis(&self, dim: usize) -> CoordPath<R> {
        assert!(
            dim < D,
            "CoordCube::axis: dim {} out of range [0, {})",
            dim,
            D
        );
        let start = dim * R;
        // SAFETY: 0 < N_VALID (11172), always valid.
        // Using new_unchecked avoids runtime branch in the hot axis path.
        let init = unsafe { Coord::new_unchecked(0) };
        let mut coords = [init; R];
        let mut i = 0;
        while i < R {
            coords[i] = self.path.coords()[start + i];
            i += 1;
        }
        CoordPath::new(coords)
    }

    /// Returns the `Coord` at a specific character within a dimension.
    ///
    /// # Panics
    ///
    /// Panics if `dim >= D` or `character >= R`.
    pub fn coord_at(&self, dim: usize, character: usize) -> Coord {
        assert!(
            dim < D,
            "CoordCube::coord_at: dim {} out of range [0, {})",
            dim,
            D
        );
        assert!(
            character < R,
            "CoordCube::coord_at: character {} out of range [0, {})",
            character,
            R
        );
        self.path.coords()[dim * R + character]
    }

    /// Returns a reference to the full coordinate array.
    #[inline]
    pub const fn coords(&self) -> &[Coord; N] {
        self.path.coords()
    }
}

// ---------------------------------------------------------------------------
// Display
// ---------------------------------------------------------------------------

impl<const N: usize, const D: usize, const R: usize> core::fmt::Display for CoordCube<N, D, R> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CoordCube<{}, {}, {}>[", N, D, R)?;
        for dim in 0..D {
            if dim > 0 {
                write!(f, " | ")?;
            }
            write!(f, "(")?;
            for i in 0..R {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", self.path.coords()[dim * R + i])?;
            }
            write!(f, ")")?;
        }
        write!(f, "]")
    }
}

// ---------------------------------------------------------------------------
// Equality (delegates to path equality)
// ---------------------------------------------------------------------------

impl<const N: usize, const D: usize, const R: usize> PartialEq for CoordCube<N, D, R> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl<const N: usize, const D: usize, const R: usize> Eq for CoordCube<N, D, R> {}

// ---------------------------------------------------------------------------
// Conversions
// ---------------------------------------------------------------------------

impl<const N: usize, const D: usize, const R: usize> From<CoordPath<N>> for CoordCube<N, D, R> {
    fn from(path: CoordPath<N>) -> Self {
        Self::from_path(path)
    }
}

impl<const N: usize, const D: usize, const R: usize> From<CoordCube<N, D, R>> for CoordPath<N> {
    fn from(cube: CoordCube<N, D, R>) -> Self {
        cube.path
    }
}
