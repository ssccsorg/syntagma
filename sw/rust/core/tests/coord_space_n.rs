use tagma_core::{
    Coord, CoordPath, CoordSpace12, CoordSpace19, CoordSpace2, CoordSpace6, CoordSpaceN,
};

// ── CoordSpaceN<1, _> — flat map tests ──

#[test]
fn new_map_is_empty() {
    let map: CoordSpaceN<1, u32> = CoordSpaceN::new();
    assert!(map.is_empty());
    assert_eq!(map.len(), 0);
    assert_eq!(map.capacity(), Some(11172));
}

#[test]
fn insert_and_get() {
    let mut map = CoordSpaceN::<1, u32>::new();
    let c = Coord::new(0).unwrap();
    assert_eq!(map.place(c, 42), None);
    assert_eq!(map.at(&c), Some(&42));
    assert_eq!(map.len(), 1);
}

#[test]
fn insert_overwrite() {
    let mut map = CoordSpaceN::<1, u32>::new();
    let c = Coord::new(0).unwrap();
    map.place(c, 1);
    assert_eq!(map.place(c, 2), Some(1));
    assert_eq!(map.at(&c), Some(&2));
    assert_eq!(map.len(), 1);
}

#[test]
fn vacate() {
    let mut map = CoordSpaceN::<1, u32>::new();
    let c = Coord::new(0).unwrap();
    map.place(c, 42);
    assert_eq!(map.vacate(&c), Some(42));
    assert_eq!(map.at(&c), None);
    assert!(map.is_empty());
}

#[test]
fn contains_key() {
    let mut map = CoordSpaceN::<1, ()>::new();
    let c = Coord::new(0).unwrap();
    assert!(!map.occupied(&c));
    map.place(c, ());
    assert!(map.occupied(&c));
}

#[test]
fn clear() {
    let mut map = CoordSpaceN::<1, u32>::new();
    map.place(Coord::new(0).unwrap(), 1);
    map.place(Coord::new(100).unwrap(), 2);
    map.clear();
    assert!(map.is_empty());
    assert_eq!(map.len(), 0);
}

#[test]
fn iter_empty() {
    let map: CoordSpaceN<1, u32> = CoordSpaceN::new();
    assert_eq!(map.iter_flat().count(), 0);
}

#[test]
fn iter_non_empty() {
    let mut map = CoordSpaceN::<1, u32>::new();
    let c1 = Coord::new(0).unwrap();
    let c2 = Coord::new(9999).unwrap();
    map.place(c1, 10);
    map.place(c2, 20);
    let entries: Vec<_> = map.iter_flat().collect();
    assert_eq!(entries.len(), 2);
    assert!(entries.contains(&(c1, &10)));
    assert!(entries.contains(&(c2, &20)));
}

#[test]
fn into_iter_consuming() {
    let mut map = CoordSpaceN::<1, &str>::new();
    let c = Coord::new(42).unwrap();
    map.place(c, "hello");
    let collected: Vec<_> = map.into_iter().collect();
    assert_eq!(collected.len(), 1);
    assert_eq!(collected[0].0, c);
    assert_eq!(collected[0].1, "hello");
}

#[test]
fn from_iterator() {
    let pairs: Vec<_> = (0..5u16)
        .map(|i| (Coord::new(i).unwrap(), i as u64))
        .collect();
    let map: CoordSpaceN<1, u64> = pairs.into_iter().collect();
    assert_eq!(map.len(), 5);
}

#[test]
fn entry_or_insert() {
    let mut map = CoordSpaceN::<1, u32>::new();
    let c = Coord::new(0).unwrap();
    map.entry(c).or_insert(42);
    assert_eq!(map.at(&c), Some(&42));
    map.entry(c).or_insert(99);
    assert_eq!(map.at(&c), Some(&42));
}

#[test]
fn entry_and_modify() {
    let mut map = CoordSpaceN::<1, u32>::new();
    let c = Coord::new(0).unwrap();
    map.entry(c).and_modify(|v| *v += 1).or_insert(1);
    assert_eq!(map.at(&c), Some(&1));
    map.entry(c).and_modify(|v| *v += 1).or_insert(1);
    assert_eq!(map.at(&c), Some(&2));
}

#[test]
fn index_trait() {
    let mut map = CoordSpaceN::<1, u32>::new();
    let c = Coord::new(5).unwrap();
    map.place(c, 42);
    assert_eq!(map[c], 42);
    map[c] = 99;
    assert_eq!(map[c], 99);
}

#[test]
fn default_is_empty() {
    let map: CoordSpaceN<1, u32> = Default::default();
    assert!(map.is_empty());
}

// ── CoordSpaceN<1, _> — path API ──

#[test]
fn flat_get_path() {
    let mut map = CoordSpaceN::<1, u32>::new();
    let c = Coord::new(42).unwrap();
    map.place(c, 100);
    assert_eq!(map.at_path(&CoordPath::new([c])), Some(&100));
}

#[test]
fn flat_insert_path() {
    let mut map = CoordSpaceN::<1, u32>::new();
    let c = Coord::new(42).unwrap();
    map.place_path(&CoordPath::new([c]), 100);
    assert_eq!(map.at(&c), Some(&100));
}

#[test]
fn flat_remove_path() {
    let mut map = CoordSpaceN::<1, u32>::new();
    let c = Coord::new(42).unwrap();
    map.place(c, 100);
    assert_eq!(map.vacate_path(&CoordPath::new([c])), Some(100));
    assert!(map.is_empty());
}

// ── CoordSpaceN<2, _> — tree map (N=2) ──

#[test]
fn tree2_insert_and_get() {
    let mut map = CoordSpaceN::<2, u32>::new();
    let c0 = Coord::new(0).unwrap();
    let c1 = Coord::new(1).unwrap();
    let path = CoordPath::new([c0, c1]);
    assert_eq!(map.place_path(&path, 42), None);
    assert_eq!(map.at_path(&path), Some(&42));
    assert_eq!(map.len(), 1);
}

#[test]
fn tree2_insert_overwrite() {
    let mut map = CoordSpaceN::<2, u32>::new();
    let path = CoordPath::new([Coord::new(0).unwrap(), Coord::new(1).unwrap()]);
    map.place_path(&path, 1);
    assert_eq!(map.place_path(&path, 2), Some(1));
    assert_eq!(map.at_path(&path), Some(&2));
    assert_eq!(map.len(), 1);
}

#[test]
fn tree2_remove() {
    let mut map = CoordSpaceN::<2, u32>::new();
    let path = CoordPath::new([Coord::new(0).unwrap(), Coord::new(1).unwrap()]);
    map.place_path(&path, 42);
    assert_eq!(map.vacate_path(&path), Some(42));
    assert_eq!(map.at_path(&path), None);
    assert!(map.is_empty());
}

#[test]
fn tree2_independent_paths() {
    let mut map = CoordSpaceN::<2, u32>::new();
    let path_a = CoordPath::new([Coord::new(0).unwrap(), Coord::new(0).unwrap()]);
    let path_b = CoordPath::new([Coord::new(0).unwrap(), Coord::new(1).unwrap()]);
    map.place_path(&path_a, 10);
    map.place_path(&path_b, 20);
    assert_eq!(map.len(), 2);
    assert_eq!(map.at_path(&path_a), Some(&10));
    assert_eq!(map.at_path(&path_b), Some(&20));
}

#[test]
fn tree2_remaining_paths_after_remove() {
    let mut map = CoordSpaceN::<2, u32>::new();
    let path_a = CoordPath::new([Coord::new(0).unwrap(), Coord::new(0).unwrap()]);
    let path_b = CoordPath::new([Coord::new(0).unwrap(), Coord::new(1).unwrap()]);
    map.place_path(&path_a, 10);
    map.place_path(&path_b, 20);
    map.vacate_path(&path_a);
    assert_eq!(map.len(), 1);
    assert_eq!(map.at_path(&path_a), None);
    assert_eq!(map.at_path(&path_b), Some(&20));
}

// ── CoordSpaceN<6, _> — UUID-scale ──

#[test]
fn tree6_basic() {
    let mut map = CoordSpaceN::<6, String>::new();
    let coords = [
        Coord::new(0).unwrap(),
        Coord::new(1).unwrap(),
        Coord::new(2).unwrap(),
        Coord::new(3).unwrap(),
        Coord::new(4).unwrap(),
        Coord::new(5).unwrap(),
    ];
    let path = CoordPath::new(coords);
    map.place_path(&path, "hello".to_string());
    assert_eq!(map.at_path(&path).map(|s| s.as_str()), Some("hello"));
}

#[test]
fn tree6_missing_path() {
    let map = CoordSpaceN::<6, u32>::new();
    let path = CoordPath::new([
        Coord::new(0).unwrap(),
        Coord::new(0).unwrap(),
        Coord::new(0).unwrap(),
        Coord::new(0).unwrap(),
        Coord::new(0).unwrap(),
        Coord::new(0).unwrap(),
    ]);
    assert_eq!(map.at_path(&path), None);
}

// ── Type aliases ──

#[test]
fn type_aliases_exist() {
    let _m1: CoordSpaceN<1, u32> = CoordSpaceN::new();
    let _m2: CoordSpace2<u32> = CoordSpaceN::new();
    let _m6: CoordSpace6<u32> = CoordSpaceN::new();
    let _m12: CoordSpace12<u32> = CoordSpaceN::new();
    let _m19: CoordSpace19<u32> = CoordSpaceN::new();
}

#[test]
fn coord_space12_basic() {
    let mut space: CoordSpace12<String> = CoordSpaceN::new();
    let path = CoordPath::new(core::array::from_fn(|i| Coord::new(i as u16).unwrap()));
    space.place_path(&path, "hello".to_string());
    assert_eq!(space.at_path(&path).map(|s| s.as_str()), Some("hello"));
    assert_eq!(space.len(), 1);
    assert_eq!(space.vacate_path(&path), Some("hello".to_string()));
    assert!(space.is_empty());
}

// ── Clear for N>1 ──

#[test]
fn tree2_clear() {
    let mut map = CoordSpaceN::<2, u32>::new();
    map.place_path(
        &CoordPath::new([Coord::new(0).unwrap(), Coord::new(0).unwrap()]),
        1,
    );
    map.place_path(
        &CoordPath::new([Coord::new(1).unwrap(), Coord::new(1).unwrap()]),
        2,
    );
    assert_eq!(map.len(), 2);
    map.clear();
    assert!(map.is_empty());
    assert_eq!(map.len(), 0);
    // Reuse after clear
    map.place_path(
        &CoordPath::new([Coord::new(2).unwrap(), Coord::new(2).unwrap()]),
        3,
    );
    assert_eq!(
        map.at_path(&CoordPath::new([
            Coord::new(2).unwrap(),
            Coord::new(2).unwrap()
        ])),
        Some(&3)
    );
}

#[test]
fn tree6_clear() {
    let mut map = CoordSpaceN::<6, u32>::new();
    let path = CoordPath::new(core::array::from_fn(|i| Coord::new(i as u16).unwrap()));
    map.place_path(&path, 42);
    assert_eq!(map.len(), 1);
    map.clear();
    assert!(map.is_empty());
}

// ── Clone / PartialEq / Debug ──

#[test]
fn tree2_clone_independent() {
    let mut a = CoordSpaceN::<2, u32>::new();
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
    assert_eq!(
        a.at_path(&CoordPath::new([
            Coord::new(0).unwrap(),
            Coord::new(0).unwrap()
        ])),
        Some(&1)
    );
    assert_eq!(
        b.at_path(&CoordPath::new([
            Coord::new(0).unwrap(),
            Coord::new(0).unwrap()
        ])),
        Some(&1)
    );
    assert_eq!(
        b.at_path(&CoordPath::new([
            Coord::new(1).unwrap(),
            Coord::new(1).unwrap()
        ])),
        Some(&2)
    );
}

#[test]
fn tree2_partial_eq() {
    let mut a = CoordSpaceN::<2, u32>::new();
    let mut b = CoordSpaceN::<2, u32>::new();
    let p = CoordPath::new([Coord::new(0).unwrap(), Coord::new(0).unwrap()]);
    a.place_path(&p, 42);
    b.place_path(&p, 42);
    assert_eq!(a, b);
    b.place_path(
        &CoordPath::new([Coord::new(1).unwrap(), Coord::new(1).unwrap()]),
        99,
    );
    assert_ne!(a, b);
}

#[test]
fn tree2_debug_format() {
    let mut map = CoordSpaceN::<2, u32>::new();
    map.place_path(
        &CoordPath::new([Coord::new(0).unwrap(), Coord::new(0).unwrap()]),
        1,
    );
    let s = format!("{:?}", map);
    assert!(s.contains("CoordSpace"));
    assert!(s.contains("N: 2"));
    assert!(s.contains("len: 1"));
}

#[test]
fn coord_space1_is_coord_space_1() {
    let mut s1: CoordSpaceN<1, u32> = CoordSpaceN::new();
    let c = Coord::new(0).unwrap();
    s1.place(c, 42);
    assert_eq!(s1.at(&c), Some(&42));
}

#[test]
fn coord_space6_uuid_scale() {
    let mut space: CoordSpace6<u32> = CoordSpaceN::new();
    let path = CoordPath::new([
        Coord::new(0).unwrap(),
        Coord::new(0).unwrap(),
        Coord::new(0).unwrap(),
        Coord::new(0).unwrap(),
        Coord::new(0).unwrap(),
        Coord::new(0).unwrap(),
    ]);
    space.place_path(&path, 42);
    assert_eq!(space.at_path(&path), Some(&42));
}

#[test]
fn max_depth_insert() {
    let mut map = CoordSpaceN::<19, u32>::new();
    let coords = [
        Coord::new(0).unwrap(),
        Coord::new(1).unwrap(),
        Coord::new(2).unwrap(),
        Coord::new(3).unwrap(),
        Coord::new(4).unwrap(),
        Coord::new(5).unwrap(),
        Coord::new(6).unwrap(),
        Coord::new(7).unwrap(),
        Coord::new(8).unwrap(),
        Coord::new(9).unwrap(),
        Coord::new(10).unwrap(),
        Coord::new(11).unwrap(),
        Coord::new(12).unwrap(),
        Coord::new(13).unwrap(),
        Coord::new(14).unwrap(),
        Coord::new(15).unwrap(),
        Coord::new(16).unwrap(),
        Coord::new(17).unwrap(),
        Coord::new(18).unwrap(),
    ];
    let path = CoordPath::new(coords);
    map.place_path(&path, 42);
    assert_eq!(map.at_path(&path), Some(&42));
    assert_eq!(map.len(), 1);
}

#[test]
fn iter_prefix_test() {
    let mut space: CoordSpaceN<2, u32> = CoordSpaceN::new();
    space.place_path(
        &CoordPath::new([Coord::new(42).unwrap(), Coord::new(1).unwrap()]),
        10,
    );
    space.place_path(
        &CoordPath::new([Coord::new(42).unwrap(), Coord::new(2).unwrap()]),
        20,
    );
    space.place_path(
        &CoordPath::new([Coord::new(99).unwrap(), Coord::new(0).unwrap()]),
        30,
    );
    let prefix = [Coord::new(42).unwrap()];
    let mut results: Vec<_> = space.iter_prefix(&prefix).unwrap().collect();
    results.sort_by_key(|(_, v)| *v);
    assert_eq!(results.len(), 2);
    assert_eq!(*results[0].1, 10);
    assert_eq!(*results[1].1, 20);
    let missing = [Coord::new(11111).unwrap()];
    assert!(space.iter_prefix(&missing).is_none());
}

// ── TreeIter lazy DFS coverage ─────────────────────────────────────

#[test]
fn iter_tree_yields_all_entries() {
    let mut space: CoordSpaceN<2, u32> = CoordSpaceN::new();
    let paths: Vec<CoordPath<2>> = (0u16..50)
        .map(|i| CoordPath::new([Coord::new(i).unwrap(), Coord::new(i + 100).unwrap()]))
        .collect();
    for (i, p) in paths.iter().enumerate() {
        space.place_path(p, i as u32);
    }

    let count = space.iter_tree().count();
    assert_eq!(count, 50, "iter_tree must yield all entries");
}

#[test]
fn iter_tree_paths_match_at_path() {
    let mut space: CoordSpaceN<2, u32> = CoordSpaceN::new();
    let inserted: Vec<(CoordPath<2>, u32)> = (0u16..50)
        .map(|i| {
            let p = CoordPath::new([Coord::new(i).unwrap(), Coord::new(i + 100).unwrap()]);
            space.place_path(&p, i as u32);
            (p, i as u32)
        })
        .collect();

    for (path, val) in space.iter_tree() {
        // Every entry from iter_tree must match at_path
        assert_eq!(space.at_path(&path), Some(val));
        // And must be one of the inserted values
        assert!(inserted.iter().any(|(p, v)| p == &path && v == val));
    }
}

#[test]
fn iter_tree_order_is_deterministic() {
    let mut space: CoordSpaceN<2, u32> = CoordSpaceN::new();
    // Insert in reverse order
    for i in (0u16..100).rev() {
        let p = CoordPath::new([Coord::new(i).unwrap(), Coord::new(0).unwrap()]);
        space.place_path(&p, i as u32);
    }
    // iter_tree must yield in ascending coordinate order (depth-first)
    let mut last = 0u16;
    for (path, _) in space.iter_tree() {
        assert!(
            path.coords()[0].index() >= last,
            "iter_tree must be in ascending coord order"
        );
        last = path.coords()[0].index();
    }
}

#[test]
fn iter_tree_produces_same_entries_as_count() {
    // Verify tree with mixed depths and values.
    let mut space: CoordSpaceN<3, u32> = CoordSpaceN::new();
    for i in 0u16..5 {
        for j in 0u16..3 {
            let p = CoordPath::new([
                Coord::new(i).unwrap(),
                Coord::new(j).unwrap(),
                Coord::new(0).unwrap(),
            ]);
            space.place_path(&p, (i * 3 + j) as u32);
        }
    }
    let count_from_len = space.len();
    let count_from_iter = space.iter_tree().count();
    assert_eq!(
        count_from_len, count_from_iter,
        "len() and iter_tree().count() must agree"
    );
}

// ── iter_prefix coverage ───────────────────────────────────────────

#[test]
fn iter_prefix_yields_correct_subset() {
    let mut space: CoordSpaceN<2, u32> = CoordSpaceN::new();
    space.place_path(
        &CoordPath::new([Coord::new(1).unwrap(), Coord::new(1).unwrap()]),
        10,
    );
    space.place_path(
        &CoordPath::new([Coord::new(1).unwrap(), Coord::new(2).unwrap()]),
        20,
    );
    space.place_path(
        &CoordPath::new([Coord::new(2).unwrap(), Coord::new(1).unwrap()]),
        30,
    );
    space.place_path(
        &CoordPath::new([Coord::new(2).unwrap(), Coord::new(2).unwrap()]),
        40,
    );

    // prefix [Coord(1)] should yield 2 entries
    let prefix = [Coord::new(1).unwrap()];
    let count = space.iter_prefix(&prefix).unwrap().count();
    assert_eq!(count, 2, "iter_prefix must yield only entries under prefix");

    // Entries under prefix must have matching first coord
    for (path, _) in space.iter_prefix(&prefix).unwrap() {
        assert_eq!(path.coords()[0].index(), 1);
    }
}

#[test]
fn iter_prefix_nonexistent_returns_none() {
    let mut space: CoordSpaceN<2, u32> = CoordSpaceN::new();
    space.place_path(
        &CoordPath::new([Coord::new(1).unwrap(), Coord::new(1).unwrap()]),
        10,
    );

    // prefix that doesn't exist
    assert!(space.iter_prefix(&[Coord::new(9999).unwrap()]).is_none());
}
