use tagma_core::{
    Coord, CoordPath, CoordSpace, CoordSpaceN, CoordSpaceN12, CoordSpaceN19, CoordSpaceN2,
    CoordSpaceN3, CoordSpaceN6,
};

// ── CoordSpaceN2 — cross-product FIH-like scenario ──

#[test]
fn cm2_fih_cube() {
    // Simulate Fact(11,172) × Intent(11,172) × Hint(11,172) = use CoordSpaceN2
    // as cross-product: Fact coord maps to Intent→Hint sub-map
    let mut map = CoordSpaceN2::new();
    // Insert 100 FIH tuples
    for f in 0u16..10 {
        for i in 0u16..10 {
            let fact = Coord::new(f).unwrap();
            let intent = Coord::new(i).unwrap();
            let path = CoordPath::new([fact, intent]);
            map.place_path(&path, (f * 1000 + i * 10) as u32);
        }
    }
    assert_eq!(map.len(), 100);
    // Query: Fact=5, Intent=7
    let r = map.at_path(&CoordPath::new([
        Coord::new(5).unwrap(),
        Coord::new(7).unwrap(),
    ]));
    assert_eq!(r, Some(&5070));
}

#[test]
fn cm2_sparse_coverage() {
    // Sparse population over large logical space
    let mut map = CoordSpaceN2::new();
    for i in 0u16..1000 {
        let c0 = Coord::new(i * 11 % 11172).unwrap();
        let c1 = Coord::new(i * 7 % 11172).unwrap();
        map.place_path(&CoordPath::new([c0, c1]), i);
    }
    assert_eq!(map.len(), 1000);
}

#[test]
fn cm2_clone_independent() {
    let mut a = CoordSpaceN2::new();
    a.place_path(
        &CoordPath::new([Coord::new(0).unwrap(), Coord::new(0).unwrap()]),
        1,
    );
    let mut b = a.clone();
    b.place_path(
        &CoordPath::new([Coord::new(1).unwrap(), Coord::new(1).unwrap()]),
        2,
    );
    assert_eq!(a.len(), 1);
    assert_eq!(b.len(), 2);
}

// ── CoordSpaceN3 — three-axis real-world pattern ──

#[test]
fn cm3_3d_grid() {
    let mut map = CoordSpaceN3::new();
    // 10×10×10 coordinate cube
    for x in 0u16..10 {
        for y in 0u16..10 {
            for z in 0u16..10 {
                let path = CoordPath::new([
                    Coord::new(x).unwrap(),
                    Coord::new(y).unwrap(),
                    Coord::new(z).unwrap(),
                ]);
                map.place_path(&path, (x * 100 + y * 10 + z) as u32);
            }
        }
    }
    assert_eq!(map.len(), 1000);
    // Query center
    let r = map.at_path(&CoordPath::new([
        Coord::new(5).unwrap(),
        Coord::new(5).unwrap(),
        Coord::new(5).unwrap(),
    ]));
    assert_eq!(r, Some(&555));
}

// ── CoordSpaceN6 — UUID-scale ──

#[test]
fn cm6_basic() {
    let mut map = CoordSpaceN6::new();
    let coords: [Coord; 6] = core::array::from_fn(|i| Coord::new(i as u16).unwrap());
    let path = CoordPath::new(coords);
    map.place_path(&path, 42);
    assert_eq!(map.at_path(&path), Some(&42));
    assert_eq!(map.len(), 1);
}

#[test]
fn cm6_missing_path() {
    let map: CoordSpaceN6<u32> = CoordSpaceN6::new();
    let path = CoordPath::new(core::array::from_fn(|_| Coord::new(0).unwrap()));
    assert_eq!(map.at_path(&path), None);
}

#[test]
fn cm6_fan_out() {
    // Single prefix fans out to multiple suffixes
    let mut map = CoordSpaceN6::new();
    let prefix = Coord::new(42).unwrap();
    for i in 0u16..100 {
        let mut coords = [Coord::new(0).unwrap(); 6];
        coords[0] = prefix;
        coords[5] = Coord::new(i).unwrap();
        map.place_path(&CoordPath::new(coords), i);
    }
    assert_eq!(map.len(), 100);
}

#[test]
fn cm6_iterate() {
    let mut map = CoordSpaceN6::new();
    for i in 0u16..10 {
        let mut coords = [Coord::new(0).unwrap(); 6];
        coords[0] = Coord::new(i).unwrap();
        map.place_path(&CoordPath::new(coords), i);
    }
    let count = map.iter_tree().count();
    assert_eq!(count, 10);
}

// ── CoordSpaceN19 — SHA-256-scale tree ──

#[test]
fn cm19_insert_and_get() {
    let mut map = CoordSpaceN19::new();
    let coords: [Coord; 19] = core::array::from_fn(|i| Coord::new(i as u16).unwrap());
    let path = CoordPath::new(coords);
    map.place_path(&path, 42);
    assert_eq!(map.at_path(&path), Some(&42));
    assert_eq!(map.len(), 1);
}

#[test]
fn cm19_multiple_paths() {
    let mut map = CoordSpaceN19::new();
    let make_path = |offset: u16| -> CoordPath<19> {
        let mut coords = [Coord::new(0).unwrap(); 19];
        for i in 0..19u16 {
            coords[i as usize] = Coord::new((i * 587 + offset) % 11172).unwrap();
        }
        CoordPath::new(coords)
    };
    let path_a = make_path(0);
    let path_b = make_path(7);
    map.place_path(&path_a, "first");
    map.place_path(&path_b, "second");
    assert_eq!(map.len(), 2);
    assert_eq!(map.at_path(&path_a), Some(&"first"));
    assert_eq!(map.at_path(&path_b), Some(&"second"));
}

// ── API parity with std HashMap (verification, not replacement) ──

#[test]
fn api_parity_with_hashmap() {
    use std::collections::HashMap;
    // Build both maps with identical data
    let mut coord_space = CoordSpace::new();
    let mut hash_map: HashMap<Coord, u32> = HashMap::new();
    for i in 0u16..500 {
        let c = Coord::new(i * 22 % 11172).unwrap();
        coord_space.place(c, i as u32);
        hash_map.insert(c, i as u32);
    }
    // Verify same entries
    for (k, v) in &hash_map {
        assert_eq!(coord_space.at(k), Some(v));
    }
    // Same iteration count
    assert_eq!(coord_space.len(), hash_map.len());
    // Same remove semantics
    let sample = Coord::new(0).unwrap();
    assert_eq!(coord_space.vacate(&sample), hash_map.remove(&sample));
    assert_eq!(coord_space.len(), hash_map.len());
}

// ── Consistency ──

#[test]
fn all_series_use_same_pattern() {
    let _m: CoordSpace<u32> = CoordSpace::new();
    let _mn: CoordSpaceN<2, u32> = CoordSpaceN::new();
    let _m2: CoordSpaceN2<u32> = CoordSpaceN2::new();
    let _m3: CoordSpaceN3<u32> = CoordSpaceN3::new();
    let _m6: CoordSpaceN6<u32> = CoordSpaceN6::new();
    let _m12: CoordSpaceN12<u32> = CoordSpaceN12::new();
    let _m19: CoordSpaceN19<u32> = CoordSpaceN19::new();
}

#[test]
fn cm12_insert_and_get() {
    let mut map = CoordSpaceN12::new();
    let coords: [Coord; 12] = core::array::from_fn(|i| Coord::new(i as u16).unwrap());
    let path = CoordPath::new(coords);
    map.place_path(&path, 42);
    assert_eq!(map.at_path(&path), Some(&42));
    assert_eq!(map.len(), 1);
    map.clear();
    assert!(map.is_empty());
}

// ── DynCoordSpace clear + reuse ──

#[test]
fn dyn_coord_clear_reuse() {
    use tagma_core::DynCoordSpace;
    let mut map = DynCoordSpace::new();
    map.place(&[Coord::new(0).unwrap()], 1);
    map.place(&[Coord::new(1).unwrap(), Coord::new(0).unwrap()], 2);
    map.clear();
    assert!(map.at(&[Coord::new(0).unwrap()]).is_none());
    assert!(map
        .at(&[Coord::new(1).unwrap(), Coord::new(0).unwrap()])
        .is_none());
    // Reuse after clear
    map.place(&[Coord::new(5).unwrap()], 10);
    assert_eq!(map.at(&[Coord::new(5).unwrap()]), Some(&10));
}

// ── DynCoordSpace stress test ──

#[test]
fn dyn_coord_stress_1000() {
    use tagma_core::DynCoordSpace;
    let mut map = DynCoordSpace::new();
    let mut inserted: Vec<Vec<Coord>> = Vec::new();
    // Phase 1: insert 100 random paths (depth 1..6)
    for _ in 0..100 {
        let depth = (inserted.len() % 5) + 1;
        let path: Vec<Coord> = (0..depth)
            .map(|i| Coord::new(((i * 587 + inserted.len()) % 11172) as u16).unwrap())
            .collect();
        map.place(&path, depth as u64);
        inserted.push(path);
    }
    // Phase 2: verify all inserted paths are retrievable
    for (idx, path) in inserted.iter().enumerate() {
        let expected = (idx % 5 + 1) as u64;
        assert_eq!(map.at(path), Some(&expected), "path {:?} not found", path);
    }
    // Phase 3: remove every other path
    for i in (0..inserted.len()).step_by(2) {
        assert!(map.vacate(&inserted[i]).is_some());
    }
    // Phase 4: verify remaining paths still accessible
    for i in (1..inserted.len()).step_by(2) {
        let expected = (i % 5 + 1) as u64;
        assert_eq!(map.at(&inserted[i]), Some(&expected));
    }
}

#[test]
fn space3_axis_projection() {
    // CoordSpaceN3: FIH-like space (Fact × Intent × Hint).
    // Insert a cross-product and verify axis projection queries.
    let mut space = CoordSpaceN3::new();
    // Fact: initial=0..5, Intent: initial=0..5, Hint: initial=0..5
    // Use only the 'initial' axis of each Coord for simplicity:
    // Coord(fact, 0, 0) × Coord(intent, 0, 0) × Coord(hint, 0, 0)
    for fact in 0..5u16 {
        for intent in 0..5u16 {
            for hint in 0..5u16 {
                let path = CoordPath::new([
                    Coord::from_axes(fact as u8, 0, 0).unwrap(),
                    Coord::from_axes(intent as u8, 0, 0).unwrap(),
                    Coord::from_axes(hint as u8, 0, 0).unwrap(),
                ]);
                space.place_path(&path, fact * 100 + intent * 10 + hint);
            }
        }
    }
    assert_eq!(space.len(), 125);
    // Projection: all entries where Fact=2 (first coordinate's initial=2)
    // This requires iterating all paths and filtering — no dedicated index.
    let fact_2: Vec<_> = space
        .iter_tree()
        .filter(|(path, _)| path.coords()[0].to_axes().0 == 2)
        .collect();
    assert_eq!(fact_2.len(), 25); // 5 intent × 5 hint
    for (path, val) in &fact_2 {
        assert_eq!(path.coords()[0].to_axes().0, 2);
        assert_eq!(**val / 100, 2);
    }
    // Projection: Fact=3 AND Intent=4
    let fi: Vec<_> = space
        .iter_tree()
        .filter(|(path, _)| path.coords()[0].to_axes().0 == 3 && path.coords()[1].to_axes().0 == 4)
        .collect();
    assert_eq!(fi.len(), 5); // 5 hints
    for (path, val) in &fi {
        assert_eq!(path.coords()[0].to_axes().0, 3);
        assert_eq!(path.coords()[1].to_axes().0, 4);
        assert_eq!(**val / 10 % 10, 4);
    }
}

#[test]
fn space19_deep_path_resolution() {
    // Verify that a 19-deep CoordPath resolves correctly at every level.
    // This tests the tree traversal logic across all intermediate nodes.
    let mut space = CoordSpaceN19::new();
    // Build a path where each coordinate is distinct: (0,1,2,...,18)
    let coords: [Coord; 19] = core::array::from_fn(|i| {
        Coord::from_axes((i % 19) as u8, (i * 3 % 21) as u8, (i * 7 % 28) as u8).unwrap()
    });
    let path = CoordPath::new(coords);
    space.place_path(&path, 999u64);
    assert_eq!(space.at_path(&path), Some(&999));
    // At each prefix length, verify that a shorter path does NOT resolve
    // (prefixes are not entries unless explicitly placed)
    for prefix_len in 1..19 {
        let mut prefix_coords = [Coord::new(0).unwrap(); 19];
        prefix_coords[..prefix_len].copy_from_slice(&coords[..prefix_len]);
        // Fill remaining with arbitrary values
        for pc in prefix_coords.iter_mut().skip(prefix_len) {
            *pc = Coord::new(0).unwrap();
        }
        let prefix_path = CoordPath::new(prefix_coords);
        // A prefix that differs only in the tail should NOT match the stored path
        if prefix_len < 19 {
            assert_ne!(
                space.at_path(&prefix_path),
                Some(&999),
                "prefix of length {} should not match full path",
                prefix_len
            );
        }
    }
    // Verify entry count: exactly 1
    assert_eq!(space.len(), 1);
}

#[test]
fn space19_sparse_19_paths() {
    // Insert 19 independent paths into the SHA-256-scale space.
    // Each path differs at every character position.
    let mut space = CoordSpaceN19::new();
    for seed in 0..19u16 {
        let coords: [Coord; 19] = core::array::from_fn(|i| {
            let v = (i as u16 * 587 + seed * 331) % 11172;
            Coord::new(v).unwrap()
        });
        let path = CoordPath::new(coords);
        space.place_path(&path, seed as u64);
    }
    assert_eq!(space.len(), 19);
    // Verify each path retrieves its value
    for seed in 0..19u16 {
        let coords: [Coord; 19] = core::array::from_fn(|i| {
            let v = (i as u16 * 587 + seed * 331) % 11172;
            Coord::new(v).unwrap()
        });
        let path = CoordPath::new(coords);
        assert_eq!(space.at_path(&path), Some(&(seed as u64)));
    }
}
