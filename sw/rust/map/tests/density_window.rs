//! Characterization tests for the density scenarios behind the
//! `Spatial/map_proximity` benchmark record.
//!
//! `CoordKey<2>` addresses the map store in byte space (0..255 per
//! character), while `CoordCube` proximity indexes span the full
//! 0..11171 range. `CoordKey::from_coord_path` truncates each index with
//! `as u8`, so storage-backed spatial queries operate on a wrapped 256-wide
//! window per character. These tests reproduce the exact fills used by the
//! benchmark and lock in the observable found counts (issue #59).

use tagma_core::{Coord, CoordPath};
use tagma_map::coord_cube_map::CoordCubeMap;
use tagma_map::coord_gen::CoordKey;
use tagma_map::coord_map_n::CoordMapN;
use tagma_map::CoordMapKey;

fn coord(value: u16) -> Coord {
    Coord::new(value).expect("valid coord index")
}

fn center_5000() -> CoordPath<2> {
    CoordPath::new([coord(5000), coord(5000)])
}

/// The dense benchmark fill: a 100x100 box over coordinates 4950..=5050.
/// Through `from_coord_path` the box lands on bytes 86..=186, which still
/// contains the query window bytes 134..=138, so every generated neighbor
/// hits. The chart plan records 9 hits at r=1 and 25 hits at r=2.
#[test]
fn dense_scenario_100x100_found_counts() {
    let mut map = CoordMapN::<2>::new();
    for x in 4950..=5050u16 {
        for y in 4950..=5050u16 {
            let path = CoordPath::new([coord(x), coord(y)]);
            map.insert_by_coordkey(&CoordKey::from_coord_path(&path), b"v".to_vec());
        }
    }

    let results_r1 = map.proximity::<2, 1>(&center_5000(), 1);
    assert_eq!(
        results_r1.len(),
        9,
        "dense r=1: all 9 generated neighbors present"
    );

    let results_r2 = map.proximity::<2, 1>(&center_5000(), 2);
    assert_eq!(
        results_r2.len(),
        25,
        "dense r=2: all 25 generated neighbors present"
    );
}

/// The sparse benchmark fill: nine scattered coordinates {4950, 5000, 5050}
/// in both axes. Through truncation these become bytes {86, 136, 186}. A
/// radius-1 query centered at byte 136 covers bytes 135..=137, so only the
/// center entry (136, 136) hits. The chart plan records 9 hits at 48.5 ns
/// for this scenario; the current code returns 1 (issue #59).
#[test]
fn sparse_scenario_nine_scattered_entries_found_count() {
    let mut map = CoordMapN::<2>::new();
    for &x in &[4950u16, 5000, 5050] {
        for &y in &[4950u16, 5000, 5050] {
            let key = CoordKey::new([x as u8, y as u8]);
            map.insert_by_coordkey(&key, b"v".to_vec());
        }
    }

    let results = map.proximity::<2, 1>(&center_5000(), 1);
    assert_eq!(
        results.len(),
        1,
        "byte-space window: only (136, 136) lies in 135..=137"
    );
}

/// The empty benchmark scenario: no entries, so the query returns nothing.
#[test]
fn empty_scenario_found_count() {
    let map: CoordMapN<2> = CoordMapN::new();
    let results = map.proximity::<2, 1>(&center_5000(), 1);
    assert!(results.is_empty());
}

/// `from_coord_path` truncates each index with `as u8`; index 256 and index
/// 0 share byte 0. The collision is structural: storage-backed spatial
/// queries address a 0..255 window per character (issue #59).
#[test]
fn from_coord_path_collides_at_index_256() {
    let mut map = CoordMapN::<1>::new();

    let high = CoordPath::new([coord(256)]);
    map.insert_by_coordkey(&CoordKey::from_coord_path(&high), b"high".to_vec());

    let low = CoordPath::new([coord(0)]);
    let stored = map.get_by_coordkey(&CoordKey::from_coord_path(&low));
    assert_eq!(stored, Some(b"high".to_vec()), "index 256 wraps to byte 0");

    let boundary = CoordPath::new([coord(255)]);
    assert_eq!(
        map.get_by_coordkey(&CoordKey::from_coord_path(&boundary)),
        None,
        "index 255 is byte 255, distinct from byte 0"
    );
}
