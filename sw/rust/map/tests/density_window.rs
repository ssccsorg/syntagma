//! Integration tests for the density scenarios behind the
//! `Spatial/map_proximity` benchmark record, under the explicit map
//! byte-space domain.
//!
//! The map key space is one byte per character (`CoordKey<N>`), so
//! storage-backed spatial queries operate in the per-character domain
//! `[0, 256)`. Region generation is bounded to this domain, centers and
//! range bounds outside it panic, and a radius that crosses the domain edge
//! clamps instead of wrapping onto low byte values. Issue #59.

use tagma_core::{Coord, CoordPath};
use tagma_map::coord_cube_map::CoordCubeMap;
use tagma_map::coord_gen::CoordKey;
use tagma_map::coord_map_n::CoordMapN;
use tagma_map::CoordMapKey;

fn coord(value: u16) -> Coord {
    Coord::new(value).expect("valid coord index")
}

fn center_136() -> CoordPath<2> {
    CoordPath::new([coord(136), coord(136)])
}

/// The dense benchmark fill mapped onto the byte domain: a 100x100 box
/// over bytes 86..=186 with the query center at byte 136. Every generated
/// neighbor lies inside the filled box, so the found counts match the
/// record (9 hits at r=1, 25 hits at r=2) without coordinate wrapping.
#[test]
fn dense_scenario_100x100_found_counts() {
    let mut map = CoordMapN::<2>::new();
    for x in 86..=186u16 {
        for y in 86..=186u16 {
            let path = CoordPath::new([coord(x), coord(y)]);
            map.insert_by_coordkey(&CoordKey::from_coord_path(&path), b"v".to_vec());
        }
    }

    let results_r1 = map.proximity::<2, 1>(&center_136(), 1);
    assert_eq!(
        results_r1.len(),
        9,
        "dense r=1: all 9 generated neighbors present"
    );

    let results_r2 = map.proximity::<2, 1>(&center_136(), 2);
    assert_eq!(
        results_r2.len(),
        25,
        "dense r=2: all 25 generated neighbors present"
    );
}

/// The sparse benchmark fill mapped onto the byte domain: nine scattered
/// bytes {86, 136, 186} in both axes. A radius-1 query centered at byte
/// 136 covers bytes 135..=137, so only the center entry (136, 136) hits.
/// The chart plan records 9 hits for this scenario; the bounded semantics
/// return 1 because the scatter points lie outside the radius-1 box
/// (issue #59).
#[test]
fn sparse_scenario_nine_scattered_entries_found_count() {
    let mut map = CoordMapN::<2>::new();
    for &x in &[86u16, 136, 186] {
        for &y in &[86u16, 136, 186] {
            let key = CoordKey::new([x as u8, y as u8]);
            map.insert_by_coordkey(&key, b"v".to_vec());
        }
    }

    let results = map.proximity::<2, 1>(&center_136(), 1);
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
    let results = map.proximity::<2, 1>(&center_136(), 1);
    assert!(results.is_empty());
}

/// A radius that crosses the top of the byte domain must clamp at byte
/// 255. Entries in bytes 0..=5 must not be reached through wrapping.
#[test]
fn proximity_does_not_wrap_across_byte_domain_edge() {
    let mut map = CoordMapN::<2>::new();
    for x in 0..=5u16 {
        for y in 0..=5u16 {
            let path = CoordPath::new([coord(x), coord(y)]);
            map.insert_by_coordkey(&CoordKey::from_coord_path(&path), b"low".to_vec());
        }
    }
    for x in 250..=255u16 {
        for y in 250..=255u16 {
            let path = CoordPath::new([coord(x), coord(y)]);
            map.insert_by_coordkey(&CoordKey::from_coord_path(&path), b"high".to_vec());
        }
    }

    let center = CoordPath::new([coord(255), coord(255)]);
    let results = map.proximity::<2, 1>(&center, 2);
    assert_eq!(
        results.len(),
        9,
        "radius 2 from byte 255 covers bytes 253..=255 only"
    );
    for (path, _) in &results {
        for c in path.coords() {
            assert!(
                c.index() >= 253,
                "wrapped low-byte entry leaked into the result"
            );
        }
    }
}

/// A center at or above the byte-space domain is a caller error and panics
/// instead of silently wrapping onto low bytes.
#[test]
#[should_panic(expected = "outside the map byte-space domain")]
fn proximity_center_outside_byte_domain_panics() {
    let map: CoordMapN<2> = CoordMapN::new();
    let center = CoordPath::new([coord(300), coord(300)]);
    let _ = map.proximity::<2, 1>(&center, 1);
}

/// `from_coord_path` cannot represent a character index at or above 256 and
/// panics instead of truncating.
#[test]
#[should_panic(expected = "exceeds the byte-space domain")]
fn from_coord_path_rejects_index_256() {
    let path = CoordPath::new([coord(256)]);
    let _ = CoordKey::from_coord_path(&path);
}

/// `bounding_box_range` rejects range bounds at or above the byte domain
/// before any generation runs.
#[test]
#[should_panic(expected = "outside the map byte-space domain")]
fn bounding_box_range_outside_byte_domain_panics() {
    let map: CoordMapN<2> = CoordMapN::new();
    let _ = map.bounding_box_range(&[(0, 255), (0, 300)]);
}
