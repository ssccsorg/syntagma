// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Taeho Lee

use tagma_core::{Coord, CoordPath, CoordSpace2};

#[test]
fn new_is_empty() {
    let s = CoordSpace2::<u32>::new();
    assert!(s.is_empty());
    assert_eq!(s.len(), 0);
}

#[test]
fn insert_and_get() {
    let mut s = CoordSpace2::<u32>::new();
    let p = CoordPath::new([Coord::new(0).unwrap(), Coord::new(1).unwrap()]);
    assert_eq!(s.place_path(&p, 42), None);
    assert_eq!(s.at_path(&p), Some(&42));
    assert_eq!(s.len(), 1);
}

#[test]
fn insert_overwrite() {
    let mut s = CoordSpace2::<u32>::new();
    let p = CoordPath::new([Coord::new(0).unwrap(), Coord::new(0).unwrap()]);
    s.place_path(&p, 1);
    assert_eq!(s.place_path(&p, 2), Some(1));
    assert_eq!(s.at_path(&p), Some(&2));
    assert_eq!(s.len(), 1);
}

#[test]
fn vacate() {
    let mut s = CoordSpace2::<u32>::new();
    let p = CoordPath::new([Coord::new(5).unwrap(), Coord::new(5).unwrap()]);
    s.place_path(&p, 99);
    assert_eq!(s.vacate_path(&p), Some(99));
    assert!(s.is_empty());
}

#[test]
fn clear() {
    let mut s = CoordSpace2::<u32>::new();
    s.place_path(
        &CoordPath::new([Coord::new(0).unwrap(), Coord::new(0).unwrap()]),
        1,
    );
    s.place_path(
        &CoordPath::new([Coord::new(1).unwrap(), Coord::new(2).unwrap()]),
        2,
    );
    assert_eq!(s.len(), 2);
    s.clear();
    assert!(s.is_empty());
    assert_eq!(
        s.at_path(&CoordPath::new([
            Coord::new(0).unwrap(),
            Coord::new(0).unwrap()
        ])),
        None
    );
}

#[test]
fn clone_eq() {
    let mut a = CoordSpace2::<u32>::new();
    let p = CoordPath::new([Coord::new(0).unwrap(), Coord::new(0).unwrap()]);
    a.place_path(&p, 42);
    let b = a.clone();
    assert_eq!(a, b);
    assert_eq!(b.at_path(&p), Some(&42));
}

#[test]
fn from_iterator() {
    let paths: Vec<_> = (0u16..10)
        .map(|i| {
            let p = CoordPath::new([Coord::new(i).unwrap(), Coord::new(i + 1).unwrap()]);
            (p, i as u32)
        })
        .collect();
    let s: CoordSpace2<u32> = paths.into_iter().collect();
    assert_eq!(s.len(), 10);
}

#[test]
fn third_slot_nonzero_stays_none() {
    let s = CoordSpace2::<u32>::new();
    let p = CoordPath::new([Coord::new(9999).unwrap(), Coord::new(8888).unwrap()]);
    assert_eq!(s.at_path(&p), None);
}
