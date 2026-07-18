use tagma_core::{Coord, CoordPath, CoordSet, CoordSpace};

// ── CoordSpace — no_alloc single-syllable ──

#[test]
fn cm_insert_11172_values() {
    let mut map = CoordSpace::new();
    for i in 0u16..11172 {
        assert_eq!(map.place(Coord::new(i).unwrap(), i as u32), None);
    }
    assert_eq!(map.len(), 11172);
}

#[test]
fn cm_all_11172_accessible() {
    let mut map = CoordSpace::new();
    for i in 0u16..11172 {
        map.place(Coord::new(i).unwrap(), i);
    }
    for i in 0u16..11172 {
        assert_eq!(map.at(&Coord::new(i).unwrap()), Some(&i));
    }
}

#[test]
fn cm_path_api() {
    let mut map = CoordSpace::new();
    let c = Coord::new(5555).unwrap();
    map.place(c, 100);
    assert_eq!(map.at(&c), Some(&100));
    assert_eq!(map.at_path(&CoordPath::new([c])), Some(&100));
}

#[test]
fn cm_remove_all() {
    let mut map = CoordSpace::new();
    for i in 0u16..11172 {
        map.place(Coord::new(i).unwrap(), i as u32);
    }
    for i in 0u16..11172 {
        map.vacate(&Coord::new(i).unwrap());
    }
    assert!(map.is_empty());
}

#[test]
fn cm_clear() {
    let mut map = CoordSpace::new();
    for i in 0u16..100 {
        map.place(Coord::new(i).unwrap(), i);
    }
    map.clear();
    assert!(map.is_empty());
}

#[test]
fn cm_entry_chained_pattern() {
    let mut map = CoordSpace::new();
    let c = Coord::new(0).unwrap();
    for _ in 0..5 {
        *map.entry(c).or_insert(0) += 1;
    }
    assert_eq!(map.at(&c), Some(&5));
}

#[test]
fn cm_entry_and_modify_chained() {
    let mut map = CoordSpace::new();
    let c = Coord::new(0).unwrap();
    map.entry(c).and_modify(|v| *v += 1).or_insert(1);
    assert_eq!(map.at(&c), Some(&1));
    map.entry(c).and_modify(|v| *v += 1).or_insert(1);
    assert_eq!(map.at(&c), Some(&2));
}

#[test]
fn cm_hll_pattern() {
    let mut map = CoordSpace::new();
    for i in 0u16..100 {
        map.place(Coord::new(i).unwrap(), 0u32);
    }
    for i in (0u16..100).step_by(3) {
        *map.at_mut(&Coord::new(i).unwrap()).unwrap() += 1;
    }
    assert_eq!(map.at(&Coord::new(0).unwrap()), Some(&1));
    assert_eq!(map.at(&Coord::new(1).unwrap()), Some(&0));
    assert_eq!(map.at(&Coord::new(99).unwrap()), Some(&1));
}

#[test]
fn cm_index_trait() {
    let mut map = CoordSpace::new();
    let c = Coord::new(5).unwrap();
    map.place(c, 42);
    assert_eq!(map[c], 42);
    map[c] = 99;
    assert_eq!(map[c], 99);
}

// ── CoordSet scenarios (no_alloc) ──

#[test]
fn set_basic() {
    let mut a = CoordSet::new();
    a.insert(Coord::new(0).unwrap());
    a.insert(Coord::new(11171).unwrap());
    assert!(a.contains(Coord::new(0).unwrap()));
    assert!(a.contains(Coord::new(11171).unwrap()));
    assert_eq!(a.len(), 2);
}

#[test]
fn set_operations() {
    let mut a = CoordSet::new();
    let mut b = CoordSet::new();
    for i in 0u16..100 {
        a.insert(Coord::new(i * 2).unwrap());
        b.insert(Coord::new(i * 3).unwrap());
    }
    let intersection = a.intersection(&b);
    assert!(intersection.contains(Coord::new(0).unwrap()));
    assert!(intersection.contains(Coord::new(6).unwrap()));
    assert!(intersection.contains(Coord::new(12).unwrap()));
    assert!(!intersection.contains(Coord::new(2).unwrap()));
    assert!(!intersection.contains(Coord::new(3).unwrap()));
    let union = a.union(&b);
    assert!(union.contains(Coord::new(2).unwrap()));
    assert!(union.contains(Coord::new(3).unwrap()));
}

// ── Spatial query tests (no_alloc) ──

#[test]
fn hamming_distance_proximity() {
    let mut space = CoordSpace::new();
    for i in 0..5u16 {
        for m in 0..5u16 {
            for f in 0..5u16 {
                let coord = Coord::from_axes(i as u8, m as u8, f as u8).unwrap();
                space.place(coord, i * 100 + m * 10 + f);
            }
        }
    }
    assert_eq!(space.len(), 125);
    let ref_coord = Coord::from_axes(2, 2, 2).unwrap();
    let (ref_i, ref_m, ref_f) = ref_coord.to_axes();
    let exactly: Vec<_> = space
        .coords()
        .filter(|c| {
            let (i, m, f) = c.to_axes();
            i == ref_i && m == ref_m && f == ref_f
        })
        .collect();
    assert_eq!(exactly.len(), 1);
    assert!(exactly.contains(&ref_coord));
}

#[test]
fn axis_slice_initial() {
    let mut space = CoordSpace::new();
    for i in 0..10u16 {
        for m in 0..10u16 {
            for f in 0..10u16 {
                let coord = Coord::from_axes(i as u8, m as u8, f as u8).unwrap();
                space.place(coord, (i, m, f));
            }
        }
    }
    let initial: Vec<_> = space.coords().filter(|c| c.to_axes().0 == 3).collect();
    assert_eq!(initial.len(), 100);
    for coord in &initial {
        assert_eq!(coord.to_axes().0, 3);
    }
}

#[test]
fn axis_slice_medial() {
    let mut space = CoordSpace::new();
    for i in 0..19u16 {
        for f in 0..28u16 {
            let coord = Coord::from_axes(i as u8, 7, f as u8).unwrap();
            space.place(coord, (i, f));
        }
    }
    let medial: Vec<_> = space.coords().filter(|c| c.to_axes().1 == 7).collect();
    assert_eq!(medial.len(), 19 * 28);
    for coord in &medial {
        assert_eq!(coord.to_axes().1, 7);
    }
}

#[test]
fn axis_slice_initial_range() {
    let mut space = CoordSpace::new();
    for i in 0..12u16 {
        for m in 0..10u16 {
            for f in 0..5u16 {
                let coord = Coord::from_axes(i as u8, m as u8, f as u8).unwrap();
                space.place(coord, i * 1000 + m * 100 + f);
            }
        }
    }
    assert_eq!(space.len(), 12 * 10 * 5);
    let range: Vec<Coord> = space
        .coords()
        .filter(|c| {
            let (i, _, _) = c.to_axes();
            (3..=7).contains(&i)
        })
        .collect();
    assert_eq!(range.len(), 5 * 10 * 5);
    for c in &range {
        let (i, _, _) = c.to_axes();
        assert!((3..=7).contains(&i), "initial {} out of range", i);
    }
}

// ── CoordSet spatial queries (no_alloc) ──

#[test]
fn coordset_radius_query() {
    let mut cluster = CoordSet::new();
    for i in 4..=6u16 {
        for m in 4..=6u16 {
            for f in 4..=6u16 {
                cluster.insert(Coord::from_axes(i as u8, m as u8, f as u8).unwrap());
            }
        }
    }
    assert_eq!(cluster.len(), 27);
    let line: CoordSet = cluster
        .iter()
        .filter(|c| {
            let (i, m, _) = c.to_axes();
            i == 5 && m == 5
        })
        .collect();
    assert_eq!(line.len(), 3);
    for c in line.iter() {
        let (i, m, _) = c.to_axes();
        assert_eq!(i, 5);
        assert_eq!(m, 5);
    }
    let mut set_a = CoordSet::new();
    let mut set_b = CoordSet::new();
    for i in 0..5u16 {
        set_a.insert(Coord::from_axes(i as u8, 0, 0).unwrap());
        set_b.insert(Coord::from_axes((i + 5) as u8, 0, 0).unwrap());
    }
    let combined = set_a.union(&set_b);
    assert_eq!(combined.len(), 10);
    for i in 0..10u16 {
        assert!(combined.contains(Coord::from_axes(i as u8, 0, 0).unwrap()));
    }
}

// ── CoordPath property verification (no_alloc) ──

#[test]
fn coord_path_is_not_a_key() {
    let path1 = CoordPath::new([Coord::new(0).unwrap(), Coord::new(0).unwrap()]);
    let path2 = CoordPath::new([Coord::new(0).unwrap(), Coord::new(1).unwrap()]);
    // A CoordPath is a path specifier, not a hash key.
    assert_ne!(path1, path2);
    // It dereferences to a slice for iteration.
    assert_eq!(path1.len(), 2);
    assert_eq!(path1.coords()[0], Coord::new(0).unwrap());
}
