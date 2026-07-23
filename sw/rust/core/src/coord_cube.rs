use crate::coord::Coord;
use crate::coord_path::CoordPath;

/// An interpretation layer over [`CoordPath`] that views `N` syllables
/// as a D-dimensional grid where each dimension has `R` syllables of
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
/// | `CoordPath` (1D) | Highest (per-syllable axes) | None |
/// | `CoordCube` (D-dim) | Lower (grouped axes) | Rich (distance, region, proximity) |
///
/// The total addressable space is exactly `11,172^N` in both views.
/// `CoordCube` never modifies or replaces `CoordPath` — it is an optional
/// interpretation layer over the same bytes.
///
/// # Relationship to the storage layer
///
/// `CoordCube` is a **view**, not a storage key.  Storage always uses
/// `CoordPath`; `CoordCube` provides a spatial interpretation for queries.
///
/// # Type Parameters
///
/// * `N` — Total number of syllables.
/// * `D` — Number of spatial dimensions.
/// * `R` — Number of syllables per dimension (resolution exponent).
///
/// # Example
///
/// ```
/// use tagma_core::{Coord, CoordPath, CoordCube};
///
/// // N = 6, D = 3, R = 2: 3 dimensions, 2 syllables each
/// let path = CoordPath::<6>::new([
///     Coord::new(0).unwrap(),   // dim 0, syllable 0
///     Coord::new(1).unwrap(),   // dim 0, syllable 1
///     Coord::new(2).unwrap(),   // dim 1, syllable 0
///     Coord::new(3).unwrap(),   // dim 1, syllable 1
///     Coord::new(4).unwrap(),   // dim 2, syllable 0
///     Coord::new(5).unwrap(),   // dim 2, syllable 1
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

    /// Returns the number of syllables per dimension.
    #[inline]
    pub const fn resolution(&self) -> usize {
        R
    }

    /// Returns the total number of syllables.
    #[inline]
    pub const fn total_syllables(&self) -> usize {
        N
    }

    /// Returns the `R`-syllable path for dimension `dim`.
    ///
    /// # Panics
    ///
    /// Panics if `dim >= D`.
    pub fn axis(&self, dim: usize) -> CoordPath<R> {
        assert!(
            dim < D,
            "CoordCube::axis: dim {} out of range [0, {})",
            dim, D
        );
        let start = dim * R;
        let init = unsafe { Coord::new_unchecked(0) };
        let mut coords = [init; R];
        let mut i = 0;
        while i < R {
            coords[i] = self.path.coords()[start + i];
            i += 1;
        }
        CoordPath::new(coords)
    }

    /// Returns the `Coord` at a specific syllable within a dimension.
    ///
    /// # Panics
    ///
    /// Panics if `dim >= D` or `syllable >= R`.
    pub fn coord_at(&self, dim: usize, syllable: usize) -> Coord {
        assert!(
            dim < D,
            "CoordCube::coord_at: dim {} out of range [0, {})",
            dim, D
        );
        assert!(
            syllable < R,
            "CoordCube::coord_at: syllable {} out of range [0, {})",
            syllable, R
        );
        self.path.coords()[dim * R + syllable]
    }

    /// Returns a reference to the full coordinate array.
    #[inline]
    pub const fn coords(&self) -> &[Coord; N] {
        self.path.coords()
    }
}

// ---------------------------------------------------------------------------
// Distance Metrics
// ---------------------------------------------------------------------------

impl<const N: usize, const D: usize, const R: usize> CoordCube<N, D, R> {
    /// Hamming distance: count of syllable positions that differ between
    /// `self` and `other`.
    ///
    /// ```
    /// use tagma_core::{Coord, CoordPath, CoordCube};
    ///
    /// let a = CoordCube::<2, 2, 1>::from_path(
    ///     CoordPath::new([Coord::new(0).unwrap(), Coord::new(0).unwrap()])
    /// );
    /// let b = CoordCube::<2, 2, 1>::from_path(
    ///     CoordPath::new([Coord::new(0).unwrap(), Coord::new(5).unwrap()])
    /// );
    /// assert_eq!(a.hamming_distance(&b), 1);
    /// ```
    pub fn hamming_distance(&self, other: &Self) -> usize {
        self.path
            .coords()
            .iter()
            .zip(other.path.coords().iter())
            .filter(|(a, b)| a != b)
            .count()
    }

    /// Axis-wise Hamming distance: returns a slice of length `D` where each
    /// entry is the number of differing syllables in that dimension.
    ///
    /// The result is stored in `out`, which must have length at least `D`.
    ///
    /// ```
    /// use tagma_core::{Coord, CoordPath, CoordCube};
    ///
    /// let a = CoordCube::<4, 2, 2>::from_path(CoordPath::new([
    ///     Coord::new(0).unwrap(), Coord::new(0).unwrap(),
    ///     Coord::new(0).unwrap(), Coord::new(0).unwrap(),
    /// ]));
    /// let b = CoordCube::<4, 2, 2>::from_path(CoordPath::new([
    ///     Coord::new(0).unwrap(), Coord::new(1).unwrap(),
    ///     Coord::new(2).unwrap(), Coord::new(0).unwrap(),
    /// ]));
    /// let mut out = [0usize; 2];
    /// a.hamming_distance_axes(&b, &mut out);
    /// assert_eq!(out, [1, 1]);
    /// ```
    pub fn hamming_distance_axes(&self, other: &Self, out: &mut [usize]) {
        for dim in 0..D {
            let start = dim * R;
            let mut syllable_diff = 0;
            for i in 0..R {
                if self.path.coords()[start + i] != other.path.coords()[start + i] {
                    syllable_diff += 1;
                }
            }
            out[dim] = syllable_diff;
        }
    }

    /// Normalised Euclidean distance approximation.
    ///
    /// Each dimension's R-syllable value is normalised to `[0, 1]`, then
    /// Euclidean distance is computed in D-dimensional normalised space.
    ///
    /// The result is in `[0, sqrt(D)]` where 0 = identical and
    /// `sqrt(D)` = maximally distant across all dimensions.
    ///
    /// # Note
    ///
    /// Uses a simple Newton-Raphson approximation for the square root
    /// to remain compatible with `no_std` environments.
    ///
    /// ```
    /// use tagma_core::{Coord, CoordPath, CoordCube};
    ///
    /// let a = CoordCube::<2, 2, 1>::from_path(
    ///     CoordPath::new([Coord::new(0).unwrap(), Coord::new(0).unwrap()])
    /// );
    /// let b = CoordCube::<2, 2, 1>::from_path(
    ///     CoordPath::new([Coord::new(0).unwrap(), Coord::new(5586).unwrap()])
    /// );
    /// let d = a.euclidean_distance_approx(&b);
    /// assert!((d - 0.5).abs() < 0.001, "got {}", d);
    /// ```
    pub fn euclidean_distance_approx(&self, other: &Self) -> f64 {
        let mut sum_sq = 0.0f64;
        let max_val = Self::dimension_max_value();
        for dim in 0..D {
            let v1 = self.dimension_value(dim);
            let v2 = other.dimension_value(dim);
            let diff = (v1 as f64 - v2 as f64) / max_val;
            sum_sq += diff * diff;
        }
        sqrt_approx(sum_sq)
    }

    /// Interprets the R syllables of dimension `dim` as a little-endian
    /// base-11172 integer in `[0, 11172^R)`.
    #[inline]
    fn dimension_value(&self, dim: usize) -> u64 {
        let start = dim * R;
        let mut val = 0u64;
        let mut mul = 1u64;
        for i in 0..R {
            let idx = self.path.coords()[start + i].index() as u64;
            val = val.wrapping_add(idx.wrapping_mul(mul));
            mul = mul.wrapping_mul(11172);
        }
        val
    }

    /// Maximum possible value for a single dimension (`11172^R - 1`),
    /// returned as an `f64` for normalisation.
    fn dimension_max_value() -> f64 {
        let mut max = 0u64;
        let mut mul = 1u64;
        for _ in 0..R {
            max = max.wrapping_add(11171u64.wrapping_mul(mul));
            mul = mul.wrapping_mul(11172);
        }
        max as f64
    }
}

// ---------------------------------------------------------------------------
// Newton-Raphson square root approximation (no_std compatible)
// ---------------------------------------------------------------------------

/// Approximates `sqrt(x)` using Newton-Raphson iteration.
///
/// Typically converges to within 1e-12 in 8–10 iterations for normalised
/// values in `[0, sqrt(D)]`.
fn sqrt_approx(x: f64) -> f64 {
    if x <= 0.0f64 {
        return 0.0f64;
    }
    // Initial guess: use bit manipulation (fast inverse sqrt style)
    let mut guess = x;
    for _ in 0..12 {
        guess = (guess + x / guess) * 0.5;
    }
    guess
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
