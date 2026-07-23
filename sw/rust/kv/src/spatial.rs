use tagma_core::{CoordCube, CoordPath};
use tagma_geo::spatial::SpatialOps;
use tagma_geo::BoundingBoxIter;

use crate::coord_kv_n::CoordKVN;
use crate::CoordKV2;

// ---------------------------------------------------------------------------
// Extension trait: SpatialKV
// ---------------------------------------------------------------------------

/// Extension trait for spatial queries on [`CoordKV`](crate::CoordKV)-compatible
/// stores.
///
/// These methods use [`CoordCube`] to interpret keys as multi-dimensional
/// coordinates and generate spatial query regions, then look up matching
/// entries in the store.
///
/// # Usage note
///
/// You must specify the cube interpretation (`D`, `R`) at the call site.
/// The constraint `D * R == N` is enforced at runtime.
pub trait SpatialKV<const N: usize> {
    /// Returns all entries within L∞ (Chebyshev) distance `radius` of
    /// `center`, interpreted as a `CoordCube<D, R>`.
    ///
    /// # Panics
    ///
    /// Panics if `D * R != N`.
    fn proximity<const D: usize, const R: usize>(
        &self,
        center: &CoordPath<N>,
        radius: usize,
    ) -> Vec<(CoordPath<N>, Vec<u8>)>;

    /// Returns all entries within a bounding box defined by per-syllable
    /// `(min, max)` ranges.
    fn bounding_box_range(&self, ranges: &[(u16, u16); N]) -> Vec<(CoordPath<N>, Vec<u8>)>;
}

// ── Implementations ──────────────────────────────────────────────────────

impl SpatialKV<2> for CoordKV2 {
    fn proximity<const D: usize, const R: usize>(
        &self,
        center: &CoordPath<2>,
        radius: usize,
    ) -> Vec<(CoordPath<2>, Vec<u8>)> {
        let cube = CoordCube::<2, D, R>::from_path(*center);
        let mut results = Vec::new();
        for path in cube.proximity(radius) {
            if let Some(val) = self.get_by_coordpath(&path) {
                results.push((path, val));
            }
        }
        results
    }

    fn bounding_box_range(&self, ranges: &[(u16, u16); 2]) -> Vec<(CoordPath<2>, Vec<u8>)> {
        let mut results = Vec::new();
        for path in BoundingBoxIter::<2>::new(*ranges) {
            if let Some(val) = self.get_by_coordpath(&path) {
                results.push((path, val));
            }
        }
        results
    }
}

impl<const N: usize> SpatialKV<N> for CoordKVN<N> {
    fn proximity<const D: usize, const R: usize>(
        &self,
        center: &CoordPath<N>,
        radius: usize,
    ) -> Vec<(CoordPath<N>, Vec<u8>)> {
        let cube = CoordCube::<N, D, R>::from_path(*center);
        let mut results = Vec::new();
        for path in cube.proximity(radius) {
            if let Some(val) = self.get_by_coordpath(&path) {
                results.push((path, val));
            }
        }
        results
    }

    fn bounding_box_range(&self, ranges: &[(u16, u16); N]) -> Vec<(CoordPath<N>, Vec<u8>)> {
        let mut results = Vec::new();
        for path in BoundingBoxIter::<N>::new(*ranges) {
            if let Some(val) = self.get_by_coordpath(&path) {
                results.push((path, val));
            }
        }
        results
    }
}

// ── Helpers: get_by_coordpath for KV types ────────────────────────────────

/// Internal helper: look up a `CoordPath<2>` in `CoordKV2`.
trait CoordPathLookup<const N: usize> {
    fn get_by_coordpath(&self, path: &CoordPath<N>) -> Option<Vec<u8>>;
}

impl CoordPathLookup<2> for CoordKV2 {
    fn get_by_coordpath(&self, path: &CoordPath<2>) -> Option<Vec<u8>> {
        use crate::CoordKVKey;
        let key = crate::coord_gen::CoordKey::from_coord_path(path);
        self.get_by_coordkey(&key)
    }
}

impl<const N: usize> CoordPathLookup<N> for CoordKVN<N> {
    fn get_by_coordpath(&self, path: &CoordPath<N>) -> Option<Vec<u8>> {
        use crate::CoordKVKey;
        let key = crate::coord_gen::CoordKey::from_coord_path(path);
        self.get_by_coordkey(&key)
    }
}

// ---------------------------------------------------------------------------
// Extend CoordKey to accept CoordPath
// ---------------------------------------------------------------------------

impl<const N: usize> crate::coord_gen::CoordKey<N> {
    /// Creates a `CoordKey<N>` from a `CoordPath<N>`.
    ///
    /// Each syllable's index byte is used as the key byte.
    pub fn from_coord_path(path: &CoordPath<N>) -> Self {
        let mut bytes = [0u8; N];
        for (i, coord) in path.coords().iter().enumerate() {
            bytes[i] = coord.index() as u8;
        }
        crate::coord_gen::CoordKey::new(bytes)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coord_gen::CoordKey;
    use crate::CoordKVKey;
    use tagma_core::{Coord, CoordPath};

    // ── CoordKV2 spatial tests ─────────────────────────────────────

    #[test]
    fn kv2_proximity_finds_nearby() {
        let mut kv = CoordKV2::new();

        // Insert at (5,5) and (5,6) — within radius 1 of center (5,5)
        let center_key = CoordKey::new([5, 5]);
        let nearby_key = CoordKey::new([5, 6]);
        let far_key = CoordKey::new([5, 20]);

        kv.insert_by_coordkey(&center_key, b"center".to_vec());
        kv.insert_by_coordkey(&nearby_key, b"nearby".to_vec());
        kv.insert_by_coordkey(&far_key, b"far".to_vec());

        let center_path = center_key.to_coord_path();
        let results = kv.proximity::<2, 1>(&center_path, 1);

        // Should find center and nearby, but not far
        assert_eq!(results.len(), 2);
        let found_paths: Vec<_> = results.iter().map(|(p, _)| *p).collect();
        assert!(found_paths.contains(&center_path));
        assert!(found_paths.contains(&nearby_key.to_coord_path()));
    }

    #[test]
    fn kv2_bounding_box_range() {
        let mut kv = CoordKV2::new();

        // Insert values at (5,5), (5,6), (10,10)
        kv.insert_by_coordkey(&CoordKey::new([5, 5]), b"v1".to_vec());
        kv.insert_by_coordkey(&CoordKey::new([5, 6]), b"v2".to_vec());
        kv.insert_by_coordkey(&CoordKey::new([10, 10]), b"v3".to_vec());

        // Box: syllable 0 in [4,6], syllable 1 in [5,7]
        let ranges = [(4, 6), (5, 7)];
        let results = kv.bounding_box_range(&ranges);

        // Should find (5,5) and (5,6), but not (10,10)
        assert_eq!(results.len(), 2);
    }

    // ── CoordKVN spatial tests ─────────────────────────────────────

    #[test]
    fn kvn_proximity_finds_nearby() {
        let mut kv = CoordKVN::<3>::new();

        // Insert at (5,5,5) and (5,5,6) — nearby
        let center_path = CoordPath::<3>::new([
            Coord::new(5).unwrap(),
            Coord::new(5).unwrap(),
            Coord::new(5).unwrap(),
        ]);
        let nearby_path = CoordPath::<3>::new([
            Coord::new(5).unwrap(),
            Coord::new(5).unwrap(),
            Coord::new(6).unwrap(),
        ]);
        let far_path = CoordPath::<3>::new([
            Coord::new(5).unwrap(),
            Coord::new(5).unwrap(),
            Coord::new(20).unwrap(),
        ]);

        kv.insert_by_coordkey(&CoordKey::from_coord_path(&center_path), b"center".to_vec());
        kv.insert_by_coordkey(&CoordKey::from_coord_path(&nearby_path), b"nearby".to_vec());
        kv.insert_by_coordkey(&CoordKey::from_coord_path(&far_path), b"far".to_vec());

        let results = kv.proximity::<3, 1>(&center_path, 1);

        // Should find center and nearby, but not far
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn kvn_proximity_empty_when_none_nearby() {
        let kv: CoordKVN<2> = CoordKVN::new();
        let center_path = CoordPath::<2>::new([Coord::new(5).unwrap(), Coord::new(5).unwrap()]);
        let results = kv.proximity::<2, 1>(&center_path, 1);
        assert!(results.is_empty());
    }

    #[test]
    fn kvn_bounding_box_range() {
        let mut kv = CoordKVN::<2>::new();

        kv.insert_by_coordkey(&CoordKey::new([5, 5]), b"v1".to_vec());
        kv.insert_by_coordkey(&CoordKey::new([5, 6]), b"v2".to_vec());
        kv.insert_by_coordkey(&CoordKey::new([10, 10]), b"v3".to_vec());

        let ranges = [(4u16, 6u16), (5u16, 7u16)];
        let results = kv.bounding_box_range(&ranges);

        // Should find (5,5) and (5,6), but not (10,10)
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn spatial_kv_empty_store_returns_empty() {
        let kv: CoordKVN<2> = CoordKVN::new();
        let ranges = [(0u16, 100u16), (0u16, 100u16)];
        let results = kv.bounding_box_range(&ranges);
        assert!(results.is_empty());

        let center_path = CoordPath::<2>::new([Coord::new(50).unwrap(), Coord::new(50).unwrap()]);
        let results = kv.proximity::<2, 1>(&center_path, 1);
        assert!(results.is_empty());
    }
}
