use tagma_core::{Coord, CoordCube, CoordPath};

// ---------------------------------------------------------------------------
// Bounding Box Iterator
// ---------------------------------------------------------------------------

/// An iterator that yields all `CoordPath<N>` within a multi-dimensional
/// bounding box, where each character position has a `(min, max)` range.
///
/// Uses a mixed-radix counter: O(N) per yield.
pub struct BoundingBoxIter<const N: usize> {
    ranges: [(u16, u16); N],
    current: [u16; N],
    finished: bool,
}

impl<const N: usize> BoundingBoxIter<N> {
    /// Creates a new bounding box iterator over the given per-character
    /// `(min, max)` ranges.
    ///
    /// # Panics
    ///
    /// Panics if any range is inverted (`min > max`) or out of bounds
    /// (`max >= 11172`).
    pub fn new(ranges: [(u16, u16); N]) -> Self {
        for (i, &(min, max)) in ranges.iter().enumerate() {
            assert!(
                min <= max,
                "BoundingBoxIter: range {} has min {} > max {}",
                i,
                min,
                max
            );
            assert!(
                max < 11172,
                "BoundingBoxIter: range {} has max {} >= 11172",
                i,
                max
            );
        }
        let mut current = [0u16; N];
        for (i, slot) in current.iter_mut().enumerate().take(N) {
            *slot = ranges[i].0;
        }
        BoundingBoxIter {
            ranges,
            current,
            finished: N == 0,
        }
    }

    /// Returns `true` if the bounding box is empty (no more paths).
    pub fn is_empty(&self) -> bool {
        self.finished
    }

    /// Returns the total count of paths (product of all range widths).
    ///
    /// Uses `saturating_mul` to prevent overflow for large ranges.
    pub fn count_paths(&self) -> usize {
        let mut total = 1usize;
        for &(min, max) in &self.ranges {
            let width = (max - min + 1) as usize;
            total = total.saturating_mul(width);
        }
        total
    }
}

impl<const N: usize> Iterator for BoundingBoxIter<N> {
    type Item = CoordPath<N>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        let mut coords = [Coord::new(0).unwrap(); N];
        for (i, slot) in coords.iter_mut().enumerate().take(N) {
            // SAFETY: current[i] is always < N_VALID because ranges are
            // validated to have max < 11172.
            *slot = unsafe { Coord::new_unchecked(self.current[i]) };
        }
        let result = CoordPath::new(coords);

        // Increment the mixed-radix counter.
        let mut pos = N;
        while pos > 0 {
            pos -= 1;
            if self.current[pos] < self.ranges[pos].1 {
                self.current[pos] += 1;
                for reset in (pos + 1)..N {
                    self.current[reset] = self.ranges[reset].0;
                }
                return Some(result);
            }
            self.current[pos] = self.ranges[pos].0;
        }
        self.finished = true;
        Some(result)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.finished {
            return (0, Some(0));
        }
        (0, Some(self.count_paths()))
    }
}

// ---------------------------------------------------------------------------
// Hamming-filtered iterator
// ---------------------------------------------------------------------------

/// An iterator that yields only paths within a Hamming radius of a center.
pub struct HammingFilter<const N: usize> {
    inner: BoundingBoxIter<N>,
    center: CoordPath<N>,
    max_distance: usize,
}

impl<const N: usize> HammingFilter<N> {
    /// Creates a new `HammingFilter` from an underlying bounding box iterator,
    /// a center path, and a maximum Hamming distance.
    pub fn new(inner: BoundingBoxIter<N>, center: CoordPath<N>, max_distance: usize) -> Self {
        HammingFilter {
            inner,
            center,
            max_distance,
        }
    }
}

impl<const N: usize> Iterator for HammingFilter<N> {
    type Item = CoordPath<N>;

    fn next(&mut self) -> Option<Self::Item> {
        for candidate in self.inner.by_ref() {
            let distance = candidate
                .coords()
                .iter()
                .zip(self.center.coords().iter())
                .filter(|(a, b)| a != b)
                .count();
            if distance <= self.max_distance {
                return Some(candidate);
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, self.inner.size_hint().1)
    }
}

// ---------------------------------------------------------------------------
// SpatialOps trait — spatial region generation
// ---------------------------------------------------------------------------

/// Extension trait providing spatial region generation on [`CoordCube`].
///
/// These methods generate `CoordPath` values within a spatial region;
/// they do **not** perform storage lookups.
pub trait SpatialOps<const N: usize> {
    /// Generates all `CoordPath<N>` within a bounding box defined by
    /// per-character `(min, max)` ranges.
    fn bounding_box(&self, ranges: &[(u16, u16); N]) -> BoundingBoxIter<N>;

    /// Generates all `CoordPath<N>` within an L∞ (Chebyshev) proximity
    /// radius of the cube's center.
    fn proximity(&self, radius: usize) -> BoundingBoxIter<N>;

    /// Generates all `CoordPath<N>` within a Hamming distance `radius`
    /// of the cube's center.
    fn proximity_hamming(&self, radius: usize) -> HammingFilter<N>;
}

impl<const N: usize, const D: usize, const R: usize> SpatialOps<N> for CoordCube<N, D, R> {
    fn bounding_box(&self, ranges: &[(u16, u16); N]) -> BoundingBoxIter<N> {
        BoundingBoxIter::new(*ranges)
    }

    fn proximity(&self, radius: usize) -> BoundingBoxIter<N> {
        let mut ranges = [(0u16, 0u16); N];
        for (i, slot) in ranges.iter_mut().enumerate().take(N) {
            let idx = self.coords()[i].index() as usize;
            let min = idx.saturating_sub(radius);
            let max = (idx + radius).min(11171);
            *slot = (min as u16, max as u16);
        }
        BoundingBoxIter::new(ranges)
    }

    fn proximity_hamming(&self, radius: usize) -> HammingFilter<N> {
        let bb = self.proximity(radius.max(1));
        let center = *self.as_path();
        HammingFilter::new(bb, center, radius)
    }
}

// ---------------------------------------------------------------------------
// DistanceMetrics trait — measurement between two CoordCubes
// ---------------------------------------------------------------------------

/// Extension trait providing distance metrics for [`CoordCube`].
///
/// All methods compare `self` to `other` and return a scalar distance.
///
/// # Dimension value overflow
///
/// For `R >= 5`, the per-dimension linear value exceeds `u64::MAX` and
/// distance results silently wrap.  Practical use with `R <= 4` is safe.
///
/// # Example
///
/// ```rust
/// use tagma_core::{Coord, CoordPath, CoordCube};
/// use tagma_geo::spatial::DistanceMetrics;
///
/// let a = CoordCube::<2, 2, 1>::from_path(
///     CoordPath::new([Coord::new(0).unwrap(), Coord::new(0).unwrap()])
/// );
/// let b = CoordCube::<2, 2, 1>::from_path(
///     CoordPath::new([Coord::new(0).unwrap(), Coord::new(5).unwrap()])
/// );
/// assert_eq!(a.hamming_distance(&b), 1);
/// ```
pub trait DistanceMetrics<const N: usize, const D: usize, const R: usize> {
    /// Hamming distance: count of character positions that differ.
    fn hamming_distance(&self, other: &CoordCube<N, D, R>) -> usize;

    /// Axis-wise Hamming distance: writes per-dimension differences into `out`.
    /// `out` must have length at least `D`.
    fn hamming_distance_axes(&self, other: &CoordCube<N, D, R>, out: &mut [usize]);

    /// Normalised Euclidean distance approximation.
    ///
    /// Each dimension's R-character value is normalised to `[0, 1]`, then
    /// Euclidean distance is computed in D-dimensional normalised space.
    /// Result is in `[0, sqrt(D)]`.
    ///
    /// # Note
    ///
    /// Uses a Newton-Raphson approximation for the square root to remain
    /// compatible with `no_std` environments.
    fn euclidean_distance_approx(&self, other: &CoordCube<N, D, R>) -> f64;

    /// Manhattan (L1) distance: sum of absolute differences across all
    /// character positions.
    fn manhattan_distance(&self, other: &CoordCube<N, D, R>) -> u64;
}

impl<const N: usize, const D: usize, const R: usize> DistanceMetrics<N, D, R>
    for CoordCube<N, D, R>
{
    fn hamming_distance(&self, other: &CoordCube<N, D, R>) -> usize {
        self.coords()
            .iter()
            .zip(other.coords().iter())
            .filter(|(a, b)| a != b)
            .count()
    }

    fn hamming_distance_axes(&self, other: &CoordCube<N, D, R>, out: &mut [usize]) {
        for (dim, slot) in out.iter_mut().enumerate().take(D) {
            let start = dim * R;
            let mut character_diff = 0;
            for i in 0..R {
                if self.coords()[start + i] != other.coords()[start + i] {
                    character_diff += 1;
                }
            }
            *slot = character_diff;
        }
    }

    fn euclidean_distance_approx(&self, other: &CoordCube<N, D, R>) -> f64 {
        let mut sum_sq = 0.0f64;
        let max_val = dimension_max_value::<R>() as f64;
        for dim in 0..D {
            let v1 = dimension_value::<N, D, R>(self, dim);
            let v2 = dimension_value::<N, D, R>(other, dim);
            let diff = (v1 as f64 - v2 as f64) / max_val;
            sum_sq += diff * diff;
        }
        sqrt_approx(sum_sq)
    }

    fn manhattan_distance(&self, other: &CoordCube<N, D, R>) -> u64 {
        let mut sum = 0u64;
        for dim in 0..D {
            let v1 = dimension_value::<N, D, R>(self, dim);
            let v2 = dimension_value::<N, D, R>(other, dim);
            sum += v1.abs_diff(v2);
        }
        sum
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Interprets the R characters of dimension `dim` of `cube` as a little-endian
/// base-11172 integer in `[0, 11172^R)`.
///
/// # Panics
///
/// Does not panic but silently wraps for `R >= 5` where the value exceeds
/// `u64::MAX`.
fn dimension_value<const N: usize, const D: usize, const R: usize>(
    cube: &CoordCube<N, D, R>,
    dim: usize,
) -> u64 {
    let start = dim * R;
    let mut val = 0u64;
    let mut mul = 1u64;
    for i in 0..R {
        let idx = cube.coords()[start + i].index() as u64;
        val = val.wrapping_add(idx.wrapping_mul(mul));
        mul = mul.wrapping_mul(11172);
    }
    val
}

/// Maximum possible value for a single dimension (`11172^R - 1`).
///
/// Returns `0` for `R >= 5` due to `u64` overflow of `11172^R`.
fn dimension_max_value<const R: usize>() -> u64 {
    let mut max = 0u64;
    let mut mul = 1u64;
    for _ in 0..R {
        max = max.wrapping_add(11171u64.wrapping_mul(mul));
        mul = mul.wrapping_mul(11172);
    }
    max
}

/// Newton-Raphson square root approximation (no_std compatible).
fn sqrt_approx(x: f64) -> f64 {
    if x <= 0.0f64 {
        return 0.0f64;
    }
    let mut guess = x;
    for _ in 0..12 {
        guess = (guess + x / guess) * 0.5;
    }
    guess
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tagma_core::{Coord, CoordCube, CoordPath};

    // ── BoundingBoxIter ────────────────────────────────────────────

    #[test]
    fn bb_iter_single_character() {
        let iter = BoundingBoxIter::<1>::new([(5, 7)]);
        assert!(!iter.is_empty());
        let paths: Vec<_> = iter.collect();
        assert_eq!(paths.len(), 3);
        assert_eq!(paths[0].coords()[0].index(), 5);
        assert_eq!(paths[2].coords()[0].index(), 7);
    }

    #[test]
    fn bb_iter_two_characters() {
        let iter = BoundingBoxIter::<2>::new([(1, 2), (3, 4)]);
        let paths: Vec<_> = iter.collect();
        assert_eq!(paths.len(), 4);
        assert_eq!(paths[0].coords()[0].index(), 1);
        assert_eq!(paths[0].coords()[1].index(), 3);
        assert_eq!(paths[3].coords()[0].index(), 2);
        assert_eq!(paths[3].coords()[1].index(), 4);
    }

    #[test]
    fn bb_iter_single_value() {
        let iter = BoundingBoxIter::<2>::new([(42, 42), (99, 99)]);
        assert!(!iter.is_empty());
        let paths: Vec<_> = iter.collect();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].coords()[0].index(), 42);
    }

    #[test]
    fn bb_iter_max_range() {
        let iter = BoundingBoxIter::<2>::new([(11170, 11171), (11171, 11171)]);
        let paths: Vec<_> = iter.collect();
        assert_eq!(paths.len(), 2);
        assert_eq!(paths[1].coords()[0].index(), 11171);
    }

    #[test]
    fn bb_iter_empty_n0() {
        let iter = BoundingBoxIter::<0>::new([]);
        assert!(iter.is_empty());
        assert!(iter.collect::<Vec<_>>().is_empty());
    }

    #[test]
    fn bb_iter_is_empty_after_exhaustion() {
        let mut iter = BoundingBoxIter::<1>::new([(0, 0)]);
        assert!(!iter.is_empty());
        let _ = iter.next();
        assert!(iter.is_empty());
    }

    #[test]
    #[should_panic(expected = "min 5 > max 3")]
    fn bb_iter_inverted_range_panics() {
        let _ = BoundingBoxIter::<1>::new([(5, 3)]);
    }

    #[test]
    #[should_panic(expected = "max 11172 >= 11172")]
    fn bb_iter_oob_range_panics() {
        let _ = BoundingBoxIter::<1>::new([(0, 11172)]);
    }

    // ── SpatialOps ─────────────────────────────────────────────────

    #[test]
    fn cube_bounding_box_basic() {
        let path = CoordPath::<2>::new([Coord::new(5).unwrap(), Coord::new(5).unwrap()]);
        let cube = CoordCube::<2, 2, 1>::from_path(path);
        let ranges = [(3u16, 6u16), (4u16, 5u16)];
        let paths: Vec<_> = cube.bounding_box(&ranges).collect();
        assert_eq!(paths.len(), 8);
    }

    #[test]
    fn cube_bounding_box_multi_character() {
        // D=2, R=2: 2-character-per-dim cube
        let path = CoordPath::<4>::new([
            Coord::new(0).unwrap(),
            Coord::new(0).unwrap(),
            Coord::new(0).unwrap(),
            Coord::new(0).unwrap(),
        ]);
        let cube = CoordCube::<4, 2, 2>::from_path(path);
        let ranges = [(0u16, 1u16), (0u16, 0u16), (0u16, 1u16), (0u16, 0u16)];
        let paths: Vec<_> = cube.bounding_box(&ranges).collect();
        // dim 0: 2 paths, dim 1: 2 paths → 4 total
        assert_eq!(paths.len(), 4);
    }

    #[test]
    fn cube_proximity_radius_zero() {
        let path = CoordPath::<2>::new([Coord::new(5).unwrap(), Coord::new(5).unwrap()]);
        let cube = CoordCube::<2, 2, 1>::from_path(path);
        let paths: Vec<_> = cube.proximity(0).collect();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].coords()[0].index(), 5);
    }

    #[test]
    fn cube_proximity_radius_one() {
        let path = CoordPath::<2>::new([Coord::new(5).unwrap(), Coord::new(5).unwrap()]);
        let cube = CoordCube::<2, 2, 1>::from_path(path);
        let paths: Vec<_> = cube.proximity(1).collect();
        assert_eq!(paths.len(), 9);
    }

    #[test]
    fn cube_proximity_clamp_to_bounds() {
        let path = CoordPath::<1>::new([Coord::new(1).unwrap()]);
        let cube = CoordCube::<1, 1, 1>::from_path(path);
        let paths: Vec<_> = cube.proximity(3).collect();
        assert_eq!(paths.len(), 5);
        assert_eq!(paths[0].coords()[0].index(), 0);
        assert_eq!(paths[4].coords()[0].index(), 4);
    }

    #[test]
    fn cube_proximity_multi_character() {
        // D=2, R=2
        let path = CoordPath::<4>::new([
            Coord::new(5).unwrap(),
            Coord::new(5).unwrap(),
            Coord::new(5).unwrap(),
            Coord::new(5).unwrap(),
        ]);
        let cube = CoordCube::<4, 2, 2>::from_path(path);
        let paths: Vec<_> = cube.proximity(1).collect();
        // 4 characters, each range width 3 → 3^4 = 81
        assert_eq!(paths.len(), 81);
    }

    // ── DistanceMetrics ────────────────────────────────────────────

    #[test]
    fn hamming_identical() {
        let path = CoordPath::<3>::new([
            Coord::new(100).unwrap(),
            Coord::new(200).unwrap(),
            Coord::new(300).unwrap(),
        ]);
        let a = CoordCube::<3, 3, 1>::from_path(path);
        let b = CoordCube::<3, 3, 1>::from_path(path);
        assert_eq!(a.hamming_distance(&b), 0);
    }

    #[test]
    fn hamming_all_differ() {
        let a = CoordCube::<3, 3, 1>::from_path(CoordPath::new([
            Coord::new(0).unwrap(),
            Coord::new(0).unwrap(),
            Coord::new(0).unwrap(),
        ]));
        let b = CoordCube::<3, 3, 1>::from_path(CoordPath::new([
            Coord::new(1).unwrap(),
            Coord::new(2).unwrap(),
            Coord::new(3).unwrap(),
        ]));
        assert_eq!(a.hamming_distance(&b), 3);
    }

    #[test]
    fn hamming_axes_works() {
        let a = CoordCube::<4, 2, 2>::from_path(CoordPath::new([
            Coord::new(0).unwrap(),
            Coord::new(0).unwrap(),
            Coord::new(0).unwrap(),
            Coord::new(0).unwrap(),
        ]));
        let b = CoordCube::<4, 2, 2>::from_path(CoordPath::new([
            Coord::new(0).unwrap(),
            Coord::new(1).unwrap(),
            Coord::new(2).unwrap(),
            Coord::new(0).unwrap(),
        ]));
        let mut out = [0usize; 2];
        a.hamming_distance_axes(&b, &mut out);
        assert_eq!(out, [1, 1]);
    }

    #[test]
    fn euclidean_identical() {
        let path = CoordPath::<2>::new([Coord::new(5000).unwrap(), Coord::new(5000).unwrap()]);
        let a = CoordCube::<2, 2, 1>::from_path(path);
        let b = CoordCube::<2, 2, 1>::from_path(path);
        assert!((a.euclidean_distance_approx(&b)).abs() < 1e-10);
    }

    #[test]
    fn euclidean_max_in_one_dim() {
        let a = CoordCube::<2, 2, 1>::from_path(CoordPath::new([
            Coord::new(0).unwrap(),
            Coord::new(0).unwrap(),
        ]));
        let b = CoordCube::<2, 2, 1>::from_path(CoordPath::new([
            Coord::new(11171).unwrap(),
            Coord::new(0).unwrap(),
        ]));
        let d = a.euclidean_distance_approx(&b);
        assert!((d - 1.0).abs() < 0.001, "got {}", d);
    }

    #[test]
    fn euclidean_multi_character() {
        // D=1, R=2: single dimension with 2 characters
        // (0, 5586) in little-endian base-11172 = 0 + 5586 * 11172 ≈ half of max
        let a = CoordCube::<2, 1, 2>::from_path(CoordPath::new([
            Coord::new(0).unwrap(),
            Coord::new(0).unwrap(),
        ]));
        let b = CoordCube::<2, 1, 2>::from_path(CoordPath::new([
            Coord::new(0).unwrap(),
            Coord::new(5586).unwrap(),
        ]));
        let d = a.euclidean_distance_approx(&b);
        // Halfway in a single dimension → ~0.5
        assert!((d - 0.5).abs() < 0.01, "got {}", d);
    }

    #[test]
    fn manhattan_identical() {
        let path = CoordPath::<2>::new([Coord::new(100).unwrap(), Coord::new(200).unwrap()]);
        let a = CoordCube::<2, 2, 1>::from_path(path);
        let b = CoordCube::<2, 2, 1>::from_path(path);
        assert_eq!(a.manhattan_distance(&b), 0);
    }

    #[test]
    fn manhattan_different() {
        let a = CoordCube::<2, 2, 1>::from_path(CoordPath::new([
            Coord::new(5).unwrap(),
            Coord::new(10).unwrap(),
        ]));
        let b = CoordCube::<2, 2, 1>::from_path(CoordPath::new([
            Coord::new(5).unwrap(),
            Coord::new(20).unwrap(),
        ]));
        assert_eq!(a.manhattan_distance(&b), 10);
    }

    #[test]
    fn manhattan_multi_character() {
        // D=1, R=2
        let a = CoordCube::<2, 1, 2>::from_path(CoordPath::new([
            Coord::new(0).unwrap(),
            Coord::new(0).unwrap(),
        ]));
        let b = CoordCube::<2, 1, 2>::from_path(CoordPath::new([
            Coord::new(10).unwrap(),
            Coord::new(20).unwrap(),
        ]));
        // value diff = 10 * 1 + 20 * 11172 = 10 + 223440 = 223450
        assert_eq!(a.manhattan_distance(&b), 10 + 20 * 11172);
    }

    // ── Hamming proximity ──────────────────────────────────────────

    #[test]
    fn proximity_hamming_radius_zero() {
        let path = CoordPath::<2>::new([Coord::new(5).unwrap(), Coord::new(5).unwrap()]);
        let cube = CoordCube::<2, 2, 1>::from_path(path);
        let paths: Vec<_> = cube.proximity_hamming(0).collect();
        assert_eq!(paths.len(), 1);
    }

    #[test]
    fn proximity_hamming_radius_one() {
        let path = CoordPath::<2>::new([Coord::new(5).unwrap(), Coord::new(5).unwrap()]);
        let cube = CoordCube::<2, 2, 1>::from_path(path);
        let paths: Vec<_> = cube.proximity_hamming(1).collect();
        assert_eq!(paths.len(), 5);
    }

    // ── HammingFilter direct construction ──────────────────────────

    #[test]
    fn hamming_filter_direct() {
        let bb = BoundingBoxIter::<2>::new([(4, 6), (4, 6)]);
        let center = CoordPath::<2>::new([Coord::new(5).unwrap(), Coord::new(5).unwrap()]);
        let filter = HammingFilter::new(bb, center, 0);
        let paths: Vec<_> = filter.collect();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].coords()[0].index(), 5);
    }

    // ── count_paths ────────────────────────────────────────────────

    #[test]
    fn bb_count_paths() {
        let iter = BoundingBoxIter::<3>::new([(0, 1), (0, 2), (0, 3)]);
        assert_eq!(iter.count_paths(), 2 * 3 * 4);
    }

    #[test]
    fn bb_count_paths_large() {
        let iter = BoundingBoxIter::<2>::new([(0, 11171), (0, 11171)]);
        assert_eq!(iter.count_paths(), 124_813_584);
    }

    #[test]
    fn bb_count_paths_empty_after_iteration() {
        let mut iter = BoundingBoxIter::<2>::new([(0, 0), (0, 0)]);
        let _ = iter.next();
        assert_eq!(iter.count_paths(), 1); // count_paths unchanged
        assert!(iter.is_empty());
    }
}
