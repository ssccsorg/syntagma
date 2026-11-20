// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Taeho Lee

use tagma_core::{Coord, CoordPath, CoordSpaceM3};

#[test]
fn new_is_empty() {
    let s = CoordSpaceM3::<u32>::new();
    assert!(s.is_empty());
    assert_eq!(s.len(), 0);
}

#[test]
fn insert_and_get() {
    let mut s = CoordSpaceM3::<u32>::new();
    let p = CoordPath::new([
        Coord::new(0).unwrap(),
        Coord::new(1).unwrap(),
        Coord::new(2).unwrap(),
    ]);
    assert_eq!(s.place_path(&p, 42), None);
    assert_eq!(s.at_path(&p), Some(&42));
    assert_eq!(s.len(), 1);
}

#[test]
fn insert_overwrite() {
    let mut s = CoordSpaceM3::<u32>::new();
    let p = CoordPath::new([
        Coord::new(0).unwrap(),
        Coord::new(0).unwrap(),
        Coord::new(0).unwrap(),
    ]);
    s.place_path(&p, 1);
    assert_eq!(s.place_path(&p, 2), Some(1));
    assert_eq!(s.at_path(&p), Some(&2));
    assert_eq!(s.len(), 1);
}

#[test]
fn vacate() {
    let mut s = CoordSpaceM3::<u32>::new();
    let p = CoordPath::new([
        Coord::new(5).unwrap(),
        Coord::new(5).unwrap(),
        Coord::new(5).unwrap(),
    ]);
    s.place_path(&p, 99);
    assert_eq!(s.vacate_path(&p), Some(99));
    assert!(s.is_empty());
}

#[test]
fn clear() {
    let mut s = CoordSpaceM3::<u32>::new();
    s.place_path(
        &CoordPath::new([
            Coord::new(0).unwrap(),
            Coord::new(0).unwrap(),
            Coord::new(0).unwrap(),
        ]),
        1,
    );
    s.place_path(
        &CoordPath::new([
            Coord::new(1).unwrap(),
            Coord::new(2).unwrap(),
            Coord::new(3).unwrap(),
        ]),
        2,
    );
    assert_eq!(s.len(), 2);
    s.clear();
    assert!(s.is_empty());
    assert_eq!(
        s.at_path(&CoordPath::new([
            Coord::new(0).unwrap(),
            Coord::new(0).unwrap(),
            Coord::new(0).unwrap(),
        ])),
        None
    );
}

#[test]
fn default_is_empty() {
    let s: CoordSpaceM3<u32> = Default::default();
    assert!(s.is_empty());
}

#[test]
fn debug_format() {
    let s = CoordSpaceM3::<u32>::new();
    let d = format!("{:?}", s);
    assert!(d.contains("CoordSpaceM"));
    assert!(d.contains("len: 0"));
}

#[test]
fn multiple_coords() {
    let mut s = CoordSpaceM3::<u32>::new();
    for i in 0u16..100 {
        let p = CoordPath::new([
            Coord::new(i).unwrap(),
            Coord::new(i + 1).unwrap(),
            Coord::new(i + 2).unwrap(),
        ]);
        s.place_path(&p, i as u32);
    }
    assert_eq!(s.len(), 100);
    for i in 0u16..100 {
        let p = CoordPath::new([
            Coord::new(i).unwrap(),
            Coord::new(i + 1).unwrap(),
            Coord::new(i + 2).unwrap(),
        ]);
        assert_eq!(s.at_path(&p), Some(&(i as u32)));
    }
}
