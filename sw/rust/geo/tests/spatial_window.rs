//! Integration tests for the bounded per-character domain in
//! `SpatialOps::proximity_bounded`.
//!
//! Region generation is clamped to a caller-defined domain `[0, domain)`.
//! A store whose keys occupy a narrower domain than the full `Coord` index
//! space (for example a byte-space store over `[0, 256)`) must bound
//! generation, otherwise a radius that crosses the domain edge wraps onto
//! low index values. Issue #59.

use tagma_core::{Coord, CoordCube, CoordPath};
use tagma_geo::spatial::SpatialOps;

const BYTE_DOMAIN: u16 = 256;

fn coord(value: u16) -> Coord {
    Coord::new(value).expect("valid coord index")
}

fn cube_at(value: u16) -> CoordCube<2, 2, 1> {
    CoordCube::from_path(CoordPath::new([coord(value), coord(value)]))
}

fn assert_within_domain(iter: impl Iterator<Item = CoordPath<2>>, domain: u16) -> usize {
    let mut count = 0;
    for path in iter {
        for c in path.coords() {
            assert!(c.index() < domain, "path coordinate above domain");
        }
        count += 1;
    }
    count
}

/// A radius that crosses the domain edge must clamp at `domain - 1` and
/// must not wrap onto low index values.
#[test]
fn proximity_bounded_clamps_at_domain_edge() {
    let cube = cube_at(254);
    let count = assert_within_domain(cube.proximity_bounded(5, BYTE_DOMAIN), BYTE_DOMAIN);
    assert_eq!(
        count,
        7 * 7,
        "center 254 with radius 5 covers bytes 249..=255"
    );
}

/// Near the middle of the domain, bounded generation agrees with the
/// full-space default.
#[test]
fn proximity_bounded_mid_domain_matches_proximity() {
    let cube = cube_at(136);
    let bounded: Vec<_> = cube.proximity_bounded(1, BYTE_DOMAIN).collect();
    let full: Vec<_> = cube.proximity(1).collect();
    assert_eq!(bounded.len(), full.len());
    assert_eq!(bounded.len(), 9);
}

/// The full-space default clamps at the top of the `Coord` index space.
#[test]
fn proximity_clamps_at_full_domain_edge() {
    let cube = cube_at(11171);
    let count = assert_within_domain(cube.proximity(3), Coord::N_VALID as u16);
    assert_eq!(
        count,
        4 * 4,
        "center 11171 with radius 3 covers 11168..=11171"
    );
}

/// A center at or above the domain is a caller error and panics instead of
/// silently wrapping.
#[test]
#[should_panic(expected = "at or above domain 256")]
fn proximity_bounded_center_out_of_domain_panics() {
    let cube = cube_at(300);
    let _ = cube.proximity_bounded(1, BYTE_DOMAIN);
}

/// A zero domain is invalid.
#[test]
#[should_panic(expected = "domain 0 must be in")]
fn proximity_bounded_zero_domain_panics() {
    let cube = cube_at(100);
    let _ = cube.proximity_bounded(1, 0);
}

/// A domain larger than the `Coord` index space is invalid.
#[test]
#[should_panic(expected = "must be in")]
fn proximity_bounded_domain_above_n_valid_panics() {
    let cube = cube_at(100);
    let domain = Coord::N_VALID as u16 + 1;
    let _ = cube.proximity_bounded(1, domain);
}
