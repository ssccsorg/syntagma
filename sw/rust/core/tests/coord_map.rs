use tagma_core::{
    Coord, CoordMap, CoordMap19, CoordMap2, CoordMap3, CoordMap6, CoordPath, CoordSet,
};

// ── CoordMap — no_alloc single-syllable ──

#[test]
fn cm_insert_11172_values() {
    let mut map = CoordMap::new();
    for i in 0u16..11172 {
        assert_eq!(map.insert(Coord::new(i).unwrap(), i as u32), None);
    }
    assert_eq!(map.len(), 11172);
}

#[test]
fn cm_all_11172_accessible() {
    let mut map = CoordMap::new();
    for i in 0u16..11172 {
        map.insert(Coord::new(i).unwrap(), i);
    }
    for i in 0u16..11172 {
        assert_eq!(map.get(&Coord::new(i).unwrap()), Some(&i));
    }
}

#[test]
fn cm_path_api() {
    let mut map = CoordMap::new();
    let c = Coord::new(5555).unwrap();
    map.insert(c, 100);
    assert_eq!(map.get(&c), Some(&100));
    assert_eq!(map.get_path(&CoordPath::new([c])), Some(&100));
}

#[test]
fn cm_remove_all() {
    let mut map = CoordMap::new();
    for i in 0u16..11172 {
        map.insert(Coord::new(i).unwrap(), i as u32);
    }
    for i in 0u16..11172 {
        map.remove(&Coord::new(i).unwrap());
    }
    assert!(map.is_empty());
}

#[test]
fn cm_clear() {
    let mut map = CoordMap::new();
    for i in 0u16..100 {
        map.insert(Coord::new(i).unwrap(), i);
    }
    map.clear();
    assert!(map.is_empty());
}

#[test]
fn cm_entry_chained_pattern() {
    let mut map = CoordMap::new();
    let c = Coord::new(0).unwrap();
    // HashMap pattern: *map.entry(k).or_insert(0) += 1
    for _ in 0..5 {
        *map.entry(c).or_insert(0) += 1;
    }
    assert_eq!(map.get(&c), Some(&5));
}

#[test]
fn cm_entry_and_modify_chained() {
    let mut map = CoordMap::new();
    let c = Coord::new(0).unwrap();
    map.entry(c).and_modify(|v| *v += 1).or_insert(1);
    assert_eq!(map.get(&c), Some(&1));
    map.entry(c).and_modify(|v| *v += 1).or_insert(1);
    assert_eq!(map.get(&c), Some(&2));
}

#[test]
fn cm_hll_pattern() {
    // HyperLogLog-like: multiple coordinates map to same counter
    let mut map = CoordMap::new();
    for i in 0u16..100 {
        map.insert(Coord::new(i).unwrap(), 0u32);
    }
    // Increment counter at various coords (simulating hash buckets)
    for i in (0u16..100).step_by(3) {
        *map.get_mut(&Coord::new(i).unwrap()).unwrap() += 1;
    }
    assert_eq!(map.get(&Coord::new(0).unwrap()), Some(&1));
    assert_eq!(map.get(&Coord::new(1).unwrap()), Some(&0));
    assert_eq!(map.get(&Coord::new(2).unwrap()), Some(&0));
    assert_eq!(map.get(&Coord::new(99).unwrap()), Some(&1));
}

#[test]
fn cm_index_trait() {
    let mut map = CoordMap::new();
    let c = Coord::new(5).unwrap();
    map.insert(c, 42);
    assert_eq!(map[c], 42);
    map[c] = 99;
    assert_eq!(map[c], 99);
}

// ── CoordMap2 — cross-product FIH-like scenario ──

#[test]
fn cm2_fih_cube() {
    // Simulate Fact(11,172) × Intent(11,172) × Hint(11,172) = use CoordMap2
    // as cross-product: Fact coord maps to Intent→Hint sub-map
    let mut map = CoordMap2::new();
    // Insert 100 FIH tuples
    for f in 0u16..10 {
        for i in 0u16..10 {
            let fact = Coord::new(f).unwrap();
            let intent = Coord::new(i).unwrap();
            let path = CoordPath::new([fact, intent]);
            map.insert_path(&path, (f * 1000 + i * 10) as u32);
        }
    }
    assert_eq!(map.len(), 100);
    // Query: Fact=5, Intent=7
    let r = map.get_path(&CoordPath::new([Coord::new(5).unwrap(), Coord::new(7).unwrap()]));
    assert_eq!(r, Some(&5070));
}

#[test]
fn cm2_sparse_coverage() {
    // Sparse population over large logical space
    let mut map = CoordMap2::new();
    for i in 0u16..1000 {
        let c0 = Coord::new(i * 11 % 11172).unwrap();
        let c1 = Coord::new(i * 7 % 11172).unwrap();
        map.insert_path(&CoordPath::new([c0, c1]), i);
    }
    assert_eq!(map.len(), 1000);
}

// ── CoordMap3 — three-axis real-world pattern ──

#[test]
fn cm3_3d_grid() {
    let mut map = CoordMap3::new();
    // 10×10×10 coordinate cube
    for x in 0u16..10 {
        for y in 0u16..10 {
            for z in 0u16..10 {
                let path = CoordPath::new([
                    Coord::new(x).unwrap(),
                    Coord::new(y).unwrap(),
                    Coord::new(z).unwrap(),
                ]);
                map.insert_path(&path, (x * 100 + y * 10 + z) as u32);
            }
        }
    }
    assert_eq!(map.len(), 1000);
    // Query center
    let r = map.get_path(&CoordPath::new([
        Coord::new(5).unwrap(),
        Coord::new(5).unwrap(),
        Coord::new(5).unwrap(),
    ]));
    assert_eq!(r, Some(&555));
}

// ── CoordMap6 — UUID-scale ──

#[test]
fn cm6_basic() {
    let mut map = CoordMap6::new();
    let coords: [Coord; 6] = core::array::from_fn(|i| Coord::new(i as u16).unwrap());
    let path = CoordPath::new(coords);
    map.insert_path(&path, 42);
    assert_eq!(map.get_path(&path), Some(&42));
    assert_eq!(map.len(), 1);
}

#[test]
fn cm6_missing_path() {
    let map: CoordMap6<u32> = CoordMap6::new();
    let path = CoordPath::new(core::array::from_fn(|_| Coord::new(0).unwrap()));
    assert_eq!(map.get_path(&path), None);
}

#[test]
fn cm6_fan_out() {
    // Single prefix fans out to multiple suffixes
    let mut map = CoordMap6::new();
    let prefix = Coord::new(42).unwrap();
    for i in 0u16..100 {
        let mut coords = [Coord::new(0).unwrap(); 6];
        coords[0] = prefix;
        coords[5] = Coord::new(i).unwrap();
        map.insert_path(&CoordPath::new(coords), i);
    }
    assert_eq!(map.len(), 100);
}

// ── CoordMap19 — SHA-256-scale tree ──

#[test]
fn cm19_insert_and_get() {
    let mut map = CoordMap19::new();
    let coords: [Coord; 19] = core::array::from_fn(|i| Coord::new(i as u16).unwrap());
    let path = CoordPath::new(coords);
    map.insert_path(&path, 42);
    assert_eq!(map.get_path(&path), Some(&42));
    assert_eq!(map.len(), 1);
}

#[test]
fn cm19_multiple_paths() {
    let mut map = CoordMap19::new();
    let make_path = |offset: u16| -> CoordPath<19> {
        let mut coords = [Coord::new(0).unwrap(); 19];
        for i in 0..19u16 {
            coords[i as usize] = Coord::new((i * 587 + offset) % 11172).unwrap();
        }
        CoordPath::new(coords)
    };
    let path_a = make_path(0);
    let path_b = make_path(7);
    map.insert_path(&path_a, "first");
    map.insert_path(&path_b, "second");
    assert_eq!(map.len(), 2);
    assert_eq!(map.get_path(&path_a), Some(&"first"));
    assert_eq!(map.get_path(&path_b), Some(&"second"));
}

// ── CoordSet scenarios ──

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
    assert!(intersection.contains(Coord::new(0).unwrap()));   // 0: even × multiple of 3
    assert!(intersection.contains(Coord::new(6).unwrap()));   // 6: 2×3, 3×2
    assert!(intersection.contains(Coord::new(12).unwrap()));  // 12: 2×6, 3×4
    assert!(!intersection.contains(Coord::new(2).unwrap()));  // 2: even, not multiple of 3
    assert!(!intersection.contains(Coord::new(3).unwrap()));  // 3: multiple of 3, not even
    let union = a.union(&b);
    assert!(union.contains(Coord::new(2).unwrap()));
    assert!(union.contains(Coord::new(3).unwrap()));
}

// ── HashMap drop-in replacement pattern ──

#[test]
fn coord_map_as_hashmap_dropin() {
    use std::collections::HashMap;
    // Build both maps with identical data
    let mut coord_map = CoordMap::new();
    let mut hash_map: HashMap<Coord, u32> = HashMap::new();
    for i in 0u16..500 {
        let c = Coord::new(i * 22 % 11172).unwrap();
        coord_map.insert(c, i as u32);
        hash_map.insert(c, i as u32);
    }
    // Verify same entries
    for (k, v) in &hash_map {
        assert_eq!(coord_map.get(k), Some(v));
    }
    // Same iteration count
    assert_eq!(coord_map.len(), hash_map.len());
    // Same remove semantics
    let sample = Coord::new(0).unwrap();
    assert_eq!(coord_map.remove(&sample), hash_map.remove(&sample));
    assert_eq!(coord_map.len(), hash_map.len());
}

// ── Consistency ──

#[test]
fn all_series_use_same_pattern() {
    let _m: CoordMap<u32> = CoordMap::new();
    let _m2: CoordMap2<u32> = CoordMap2::new();
    let _m3: CoordMap3<u32> = CoordMap3::new();
    let _m6: CoordMap6<u32> = CoordMap6::new();
    let _m19: CoordMap19<u32> = CoordMap19::new();
}

#[test]
fn coord_path_is_not_a_key() {
    let path = CoordPath::<3>::new([
        Coord::new(0).unwrap(),
        Coord::new(1).unwrap(),
        Coord::new(2).unwrap(),
    ]);
    assert_eq!(path.len(), 3);
    assert!(!path.is_empty());
}
