use tagma_core::{Coord, CoordCube, CoordPath};

// ---------------------------------------------------------------------------
// Bounding Box Iterator
// ---------------------------------------------------------------------------

/// An iterator that yields all `CoordPath<N>` within a multi-dimensional
/// bounding box, where each syllable position has a `(min, max)` range.
///
/// The bounding box is a **hyper-rectangle** in syllable-index space:
/// for each syllable position `i`, the coordinate must satisfy
/// `ranges[i].0 <= coord.index() <= ranges[i].1`.
///
/// The total number of yielded paths is the product of range widths,
/// which can be enormous.  Use with caution and small ranges.
///
/// # Performance
///
/// Iteration uses a mixed-radix counter: O(N) per yield.
pub struct BoundingBoxIter<const N: usize> {
    /// (min, max) inclusive for each syllable position.
    ranges: [(u16, u16); N],
    /// Current counter value (one per syllable position).
    current: [u16; N],
    /// Whether iteration is complete.
    finished: bool,
}

impl<const N: usize> BoundingBoxIter<N> {
    /// Creates a new bounding box iterator over the given per-syllable
    /// `(min, max)` ranges.
    ///
    /// # Panics
    ///
    /// Panics if any range is inverted (`min > max`) or out of bounds
    /// (`max >= 11172`).
    pub fn new(ranges: [(u16, u16); N]) -> Self {
        // Validate all ranges.
        for (i, &(min, max)) in ranges.iter().enumerate() {
            assert!(
                min <= max,
                "BoundingBoxIter: range {} has min {} > max {}",
                i, min, max
            );
            assert!(
                max < 11172,
                "BoundingBoxIter: range {} has max {} >= 11172",
                i, max
            );
        }
        // Build the initial `current` from mins.
        let mut current = [0u16; N];
        let mut i = 0;
        while i < N {
            current[i] = ranges[i].0;
            i += 1;
        }
        BoundingBoxIter {
            ranges,
            current,
            finished: N == 0,
        }
    }

    /// Returns `true` if the bounding box is empty (no paths to yield).
    pub fn is_empty(&self) -> bool {
        self.finished
    }

    /// Returns the total count of paths in the bounding box.
    /// This is the product of all range widths.
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

        // Build the CoordPath from current counter.
        let mut coords = [Coord::new(0).unwrap(); N];
        for i in 0..N {
            // SAFETY: current[i] is always < N_VALID because ranges are
            // validated to have max < 11172 and current is initialised
            // from ranges and only incremented within them.
            coords[i] = unsafe { Coord::new_unchecked(self.current[i]) };
        }
        let result = CoordPath::new(coords);

        // Increment the mixed-radix counter.
        let mut pos = N;
        while pos > 0 {
            pos -= 1;
            if self.current[pos] < self.ranges[pos].1 {
                self.current[pos] += 1;
                // Reset lower-order positions to their minima.
                let mut reset = pos + 1;
                while reset < N {
                    self.current[reset] = self.ranges[reset].0;
                    reset += 1;
                }
                return Some(result);
            }
            // This position has reached its max; carry to next higher position.
            self.current[pos] = self.ranges[pos].0;
        }

        // All positions wrapped — iteration is done.
        self.finished = true;
        Some(result)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.finished {
            return (0, Some(0));
        }
        let remaining = self.count_paths();
        (0, Some(remaining))
    }
}

// ---------------------------------------------------------------------------
// SpatialOps trait — extension methods for CoordCube
// ---------------------------------------------------------------------------

/// Extension trait providing spatial query generation on [`CoordCube`].
///
/// These methods generate `CoordPath` values within a spatial region;
/// they do **not** perform storage lookups.  Combine with a KV store
/// for storage-backed spatial queries.
///
/// # Example
///
/// ```rust
/// use tagma_core::{Coord, CoordPath, CoordCube};
/// use tagma_geo::spatial::{SpatialOps, BoundingBoxIter};
///
/// let path = CoordPath::<2>::new([Coord::new(5).unwrap(), Coord::new(5).unwrap()]);
/// let cube = CoordCube::<2, 2, 1>::from_path(path);
///
/// // L∞ proximity: radius 1 → 3^2 = 9 paths
/// let nearby: Vec<_> = cube.proximity(1).collect();
/// assert_eq!(nearby.len(), 9);
/// ```
pub trait SpatialOps<const N: usize> {
    /// Generates all `CoordPath<N>` within a bounding box defined by
    /// per-syllable `(min, max)` ranges.
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
        for i in 0..N {
            let idx = self.coords()[i].index() as usize;
            let min = idx.saturating_sub(radius);
            let max = (idx + radius).min(11171);
            ranges[i] = (min as u16, max as u16);
        }
        BoundingBoxIter::new(ranges)
    }

    fn proximity_hamming(&self, radius: usize) -> HammingFilter<N> {
        let bb = self.proximity(radius.max(1));
        let center = *self.as_path();
        HammingFilter {
            inner: bb,
            center,
            max_distance: radius,
        }
    }
}

// ---------------------------------------------------------------------------
// Hamming-filtered iterator (wraps BoundingBoxIter)
// ---------------------------------------------------------------------------

/// An iterator that yields only paths within a Hamming radius of a center.
pub struct HammingFilter<const N: usize> {
    inner: BoundingBoxIter<N>,
    center: CoordPath<N>,
    max_distance: usize,
}

impl<const N: usize> Iterator for HammingFilter<N> {
    type Item = CoordPath<N>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(candidate) = self.inner.next() {
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
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tagma_core::{Coord, CoordCube, CoordPath};

    // ── BoundingBoxIter basics ──────────────────────────────────────

    #[test]
    fn bb_iter_single_syllable() {
        let iter = BoundingBoxIter::<1>::new([(5, 7)]);
        let paths: Vec<_> = iter.collect();
        assert_eq!(paths.len(), 3);
        assert_eq!(paths[0].coords()[0].index(), 5);
        assert_eq!(paths[1].coords()[0].index(), 6);
        assert_eq!(paths[2].coords()[0].index(), 7);
    }

    #[test]
    fn bb_iter_two_syllables() {
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
        let paths: Vec<_> = iter.collect();
        assert_eq!(paths.len(), 1);
    }

    #[test]
    fn bb_iter_empty_n0() {
        let iter = BoundingBoxIter::<0>::new([]);
        let paths: Vec<_> = iter.collect();
        assert!(paths.is_empty());
    }

    #[test]
    #[should_panic(expected = "min 5 > max 3")]
    fn bb_iter_inverted_range_panics() {
        let _ = BoundingBoxIter::<1>::new([(5, 3)]);
    }

    // ── CoordCube::bounding_box ───────────────────────────────────

    #[test]
    fn cube_bounding_box_basic() {
        let path =
            CoordPath::<2>::new([Coord::new(5).unwrap(), Coord::new(5).unwrap()]);
        let cube = CoordCube::<2, 2, 1>::from_path(path);
        let ranges = [(3u16, 6u16), (4u16, 5u16)];
        let paths: Vec<_> = cube.bounding_box(&ranges).collect();
        assert_eq!(paths.len(), 8);
    }

    // ── CoordCube::proximity ──────────────────────────────────────

    #[test]
    fn cube_proximity_radius_zero() {
        let path =
            CoordPath::<2>::new([Coord::new(5).unwrap(), Coord::new(5).unwrap()]);
        let cube = CoordCube::<2, 2, 1>::from_path(path);
        let paths: Vec<_> = cube.proximity(0).collect();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].coords()[0].index(), 5);
    }

    #[test]
    fn cube_proximity_radius_one() {
        let path =
            CoordPath::<2>::new([Coord::new(5).unwrap(), Coord::new(5).unwrap()]);
        let cube = CoordCube::<2, 2, 1>::from_path(path);
        let paths: Vec<_> = cube.proximity(1).collect();
        assert_eq!(paths.len(), 9);
    }

    #[test]
    fn cube_proximity_clamp_to_lower_bound() {
        let path = CoordPath::<1>::new([Coord::new(1).unwrap()]);
        let cube = CoordCube::<1, 1, 1>::from_path(path);
        let paths: Vec<_> = cube.proximity(3).collect();
        assert_eq!(paths.len(), 5);
        assert_eq!(paths[0].coords()[0].index(), 0);
        assert_eq!(paths[4].coords()[0].index(), 4);
    }

    #[test]
    fn cube_proximity_clamp_to_upper_bound() {
        let path = CoordPath::<1>::new([Coord::new(11170).unwrap()]);
        let cube = CoordCube::<1, 1, 1>::from_path(path);
        let paths: Vec<_> = cube.proximity(2).collect();
        assert_eq!(paths.len(), 4);
        assert_eq!(paths[0].coords()[0].index(), 11168);
        assert_eq!(paths[3].coords()[0].index(), 11171);
    }

    // ── Hamming proximity ─────────────────────────────────────────

    #[test]
    fn cube_proximity_hamming_radius_zero() {
        let path = CoordPath::<3>::new([
            Coord::new(0).unwrap(),
            Coord::new(0).unwrap(),
            Coord::new(0).unwrap(),
        ]);
        let cube = CoordCube::<3, 3, 1>::from_path(path);
        let paths: Vec<_> = cube.proximity_hamming(0).collect();
        assert_eq!(paths.len(), 1);
    }

    #[test]
    fn cube_proximity_hamming_radius_one() {
        let path =
            CoordPath::<2>::new([Coord::new(5).unwrap(), Coord::new(5).unwrap()]);
        let cube = CoordCube::<2, 2, 1>::from_path(path);
        let paths: Vec<_> = cube.proximity_hamming(1).collect();
        // Center (5,5) + neighbours with at most 1 differing syllable = 5
        assert_eq!(paths.len(), 5);
    }

    // ── BoundingBoxIter count_paths ───────────────────────────────

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
}
