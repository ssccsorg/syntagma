use tagma_core::{Coord, CoordPath, CoordSetN};

fn c(idx: u16) -> Coord {
    Coord::new(idx).unwrap()
}
fn path<const N: usize>(indices: &[u16]) -> CoordPath<N> {
    assert_eq!(indices.len(), N, "path length must match depth");
    let mut arr = [c(0); N];
    for (i, &idx) in indices.iter().enumerate() {
        arr[i] = c(idx);
    }
    CoordPath::new(arr)
}

#[test]
fn new_set_is_empty() {
    let s: CoordSetN<2> = CoordSetN::new();
    assert!(s.is_empty());
    assert_eq!(s.len(), 0);
}

#[test]
fn insert_and_contains() {
    let mut s = CoordSetN::<2>::new();
    let p = path(&[1, 2]);
    assert!(s.insert(p));
    assert!(s.contains(p));
    assert_eq!(s.len(), 1);
}

#[test]
fn insert_duplicate_returns_false() {
    let mut s = CoordSetN::<2>::new();
    let p = path(&[1, 2]);
    assert!(s.insert(p));
    assert!(!s.insert(p));
    assert_eq!(s.len(), 1);
}

#[test]
fn contains_nonexistent_returns_false() {
    let s: CoordSetN<2> = CoordSetN::new();
    assert!(!s.contains(path(&[0, 0])));
}

#[test]
fn clear_removes_all() {
    let mut s = CoordSetN::<2>::new();
    s.insert(path(&[1, 2]));
    s.insert(path(&[3, 4]));
    assert_eq!(s.len(), 2);
    s.clear();
    assert!(s.is_empty());
}

#[test]
fn iter_yields_all_paths() {
    let mut s = CoordSetN::<2>::new();
    s.insert(path(&[1, 2]));
    s.insert(path(&[3, 4]));
    let paths: Vec<_> = s.iter().map(|(p, _)| p).collect();
    assert_eq!(paths.len(), 2);
}

#[test]
fn union_combines_both() {
    let mut a = CoordSetN::<2>::new();
    a.insert(path(&[1, 2]));
    let mut b = CoordSetN::<2>::new();
    b.insert(path(&[3, 4]));
    let u = a.union(&b);
    assert!(u.contains(path(&[1, 2])));
    assert!(u.contains(path(&[3, 4])));
    assert_eq!(u.len(), 2);
}

#[test]
fn union_deduplicates() {
    let mut a = CoordSetN::<2>::new();
    a.insert(path(&[1, 2]));
    let mut b = CoordSetN::<2>::new();
    b.insert(path(&[1, 2]));
    b.insert(path(&[3, 4]));
    let u = a.union(&b);
    assert_eq!(u.len(), 2);
}

#[test]
fn intersection_common_only() {
    let mut a = CoordSetN::<2>::new();
    a.insert(path(&[1, 2]));
    a.insert(path(&[5, 6]));
    let mut b = CoordSetN::<2>::new();
    b.insert(path(&[1, 2]));
    b.insert(path(&[3, 4]));
    let i = a.intersection(&b);
    assert!(i.contains(path(&[1, 2])));
    assert_eq!(i.len(), 1);
}

#[test]
fn intersection_empty_when_disjoint() {
    let mut a = CoordSetN::<2>::new();
    a.insert(path(&[1, 2]));
    let mut b = CoordSetN::<2>::new();
    b.insert(path(&[3, 4]));
    let i = a.intersection(&b);
    assert!(i.is_empty());
}

#[test]
fn intersection_iterates_smaller_set() {
    let mut large = CoordSetN::<2>::new();
    for i in 0..100 {
        large.insert(path(&[i, 0]));
    }
    let mut small = CoordSetN::<2>::new();
    small.insert(path(&[1, 0]));
    let i = small.intersection(&large);
    assert_eq!(i.len(), 1);
}

#[test]
fn difference_subtracts() {
    let mut a = CoordSetN::<2>::new();
    a.insert(path(&[1, 2]));
    a.insert(path(&[5, 6]));
    let mut b = CoordSetN::<2>::new();
    b.insert(path(&[1, 2]));
    let d = a.difference(&b);
    assert!(d.contains(path(&[5, 6])));
    assert_eq!(d.len(), 1);
}

#[test]
fn symmetric_difference_exclusive_only() {
    let mut a = CoordSetN::<2>::new();
    a.insert(path(&[1, 2]));
    a.insert(path(&[3, 4]));
    let mut b = CoordSetN::<2>::new();
    b.insert(path(&[3, 4]));
    b.insert(path(&[5, 6]));
    let d = a.symmetric_difference(&b);
    assert!(d.contains(path(&[1, 2])));
    assert!(!d.contains(path(&[3, 4])));
    assert!(d.contains(path(&[5, 6])));
    assert_eq!(d.len(), 2);
}

#[test]
fn is_subset_true() {
    let mut a = CoordSetN::<2>::new();
    a.insert(path(&[1, 2]));
    let mut b = CoordSetN::<2>::new();
    b.insert(path(&[1, 2]));
    b.insert(path(&[3, 4]));
    assert!(a.is_subset(&b));
}

#[test]
fn is_subset_false() {
    let mut a = CoordSetN::<2>::new();
    a.insert(path(&[1, 2]));
    a.insert(path(&[5, 6]));
    let mut b = CoordSetN::<2>::new();
    b.insert(path(&[1, 2]));
    assert!(!a.is_subset(&b));
}

#[test]
fn is_disjoint_true() {
    let mut a = CoordSetN::<2>::new();
    a.insert(path(&[1, 2]));
    let mut b = CoordSetN::<2>::new();
    b.insert(path(&[3, 4]));
    assert!(a.is_disjoint(&b));
}

#[test]
fn is_disjoint_false() {
    let mut a = CoordSetN::<2>::new();
    a.insert(path(&[1, 2]));
    let mut b = CoordSetN::<2>::new();
    b.insert(path(&[1, 2]));
    b.insert(path(&[3, 4]));
    assert!(!a.is_disjoint(&b));
}

#[test]
fn depth_3_works() {
    let mut s = CoordSetN::<3>::new();
    let p = path(&[0, 1, 2]);
    s.insert(p);
    assert!(s.contains(p));
    assert_eq!(s.len(), 1);
}

#[test]
fn eq_same_content() {
    let mut a = CoordSetN::<2>::new();
    a.insert(path(&[1, 2]));
    a.insert(path(&[3, 4]));
    let mut b = CoordSetN::<2>::new();
    b.insert(path(&[3, 4]));
    b.insert(path(&[1, 2]));
    assert_eq!(a, b);
}

#[test]
fn eq_different_content() {
    let mut a = CoordSetN::<2>::new();
    a.insert(path(&[1, 2]));
    let mut b = CoordSetN::<2>::new();
    b.insert(path(&[3, 4]));
    assert_ne!(a, b);
}

#[test]
fn from_iterator_collects_all() {
    let paths = vec![path(&[1, 2]), path(&[3, 4])];
    let s: CoordSetN<2> = paths.into_iter().collect();
    assert_eq!(s.len(), 2);
    assert!(s.contains(path(&[1, 2])));
    assert!(s.contains(path(&[3, 4])));
}

#[test]
fn from_iterator_deduplicates() {
    let paths = vec![path(&[1, 2]), path(&[1, 2]), path(&[3, 4])];
    let s: CoordSetN<2> = paths.into_iter().collect();
    assert_eq!(s.len(), 2);
}

// ── Edge cases: empty set interactions ────────────────────────────

#[test]
fn union_with_empty_returns_self() {
    let mut a = CoordSetN::<2>::new();
    a.insert(path(&[1, 2]));
    let empty = CoordSetN::<2>::new();
    let u = a.union(&empty);
    assert_eq!(u.len(), 1);
    assert!(u.contains(path(&[1, 2])));
}

#[test]
fn intersection_with_empty_returns_empty() {
    let mut a = CoordSetN::<2>::new();
    a.insert(path(&[1, 2]));
    let empty = CoordSetN::<2>::new();
    let i = a.intersection(&empty);
    assert!(i.is_empty());
}

#[test]
fn difference_with_empty_returns_self() {
    let mut a = CoordSetN::<2>::new();
    a.insert(path(&[1, 2]));
    let empty = CoordSetN::<2>::new();
    let d = a.difference(&empty);
    assert_eq!(d.len(), 1);
}

#[test]
fn symmetric_difference_with_empty_returns_self() {
    let mut a = CoordSetN::<2>::new();
    a.insert(path(&[1, 2]));
    let empty = CoordSetN::<2>::new();
    let d = a.symmetric_difference(&empty);
    assert_eq!(d.len(), 1);
}

#[test]
fn is_subset_empty_true_for_any() {
    let mut a = CoordSetN::<2>::new();
    a.insert(path(&[1, 2]));
    let empty = CoordSetN::<2>::new();
    assert!(empty.is_subset(&a));
    assert!(!a.is_subset(&empty));
}

#[test]
fn is_disjoint_empty_true_for_any() {
    let mut a = CoordSetN::<2>::new();
    a.insert(path(&[1, 2]));
    let empty = CoordSetN::<2>::new();
    assert!(a.is_disjoint(&empty));
    assert!(empty.is_disjoint(&a));
}

#[test]
fn iter_empty_yields_nothing() {
    let s: CoordSetN<2> = CoordSetN::new();
    assert_eq!(s.iter().count(), 0);
}

#[test]
fn clear_then_reinsert() {
    let mut s = CoordSetN::<2>::new();
    s.insert(path(&[1, 2]));
    s.clear();
    assert!(s.is_empty());
    s.insert(path(&[3, 4]));
    assert!(s.contains(path(&[3, 4])));
    assert_eq!(s.len(), 1);
}

#[test]
fn remove_removes_existing() {
    let mut s = CoordSetN::<2>::new();
    s.insert(path(&[1, 2]));
    assert!(s.remove(path(&[1, 2])));
    assert!(!s.contains(path(&[1, 2])));
    assert!(s.is_empty());
}

#[test]
fn remove_nonexistent_returns_false() {
    let mut s = CoordSetN::<2>::new();
    s.insert(path(&[1, 2]));
    assert!(!s.remove(path(&[3, 4])));
    assert_eq!(s.len(), 1);
}

#[test]
fn is_superset_true() {
    let mut a = CoordSetN::<2>::new();
    a.insert(path(&[1, 2]));
    a.insert(path(&[3, 4]));
    let mut b = CoordSetN::<2>::new();
    b.insert(path(&[1, 2]));
    assert!(a.is_superset(&b));
    assert!(!b.is_superset(&a));
}

#[test]
fn is_superset_equal_sets() {
    let mut a = CoordSetN::<2>::new();
    a.insert(path(&[1, 2]));
    let mut b = CoordSetN::<2>::new();
    b.insert(path(&[1, 2]));
    assert!(a.is_superset(&b));
    assert!(b.is_superset(&a));
}

#[test]
fn iter_tree_yields_same_as_len() {
    let mut s = CoordSetN::<2>::new();
    for i in 0u16..30 {
        s.insert(CoordPath::new([
            Coord::new(i).unwrap(),
            Coord::new(i + 50).unwrap(),
        ]));
    }
    assert_eq!(s.iter().count(), 30);
}

#[test]
fn iter_tree_order_deterministic() {
    let mut s = CoordSetN::<2>::new();
    // Reverse insert order
    for i in (0u16..20).rev() {
        s.insert(CoordPath::new([
            Coord::new(i).unwrap(),
            Coord::new(0).unwrap(),
        ]));
    }
    let mut last = 0u16;
    for (path, _) in s.iter() {
        assert!(path.coords()[0].index() >= last, "iter must be ascending");
        last = path.coords()[0].index();
    }
}

#[test]
fn iter_size_hint_covers_contents() {
    let mut s = CoordSetN::<2>::new();
    for i in 0u16..10 {
        s.insert(CoordPath::new([
            Coord::new(i).unwrap(),
            Coord::new(i).unwrap(),
        ]));
    }
    // size_hint lower bound is 0 (lazy iter), upper bound None
    // But iter must still yield all entries
    let iter = s.iter();
    let (lower, _upper) = iter.size_hint();
    assert_eq!(lower, 0);
    assert_eq!(iter.count(), 10);
}
