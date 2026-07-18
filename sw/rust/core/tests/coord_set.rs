use tagma_core::{Coord, CoordSet};

#[test]
fn new_set_is_empty() {
    let set = CoordSet::new();
    assert!(set.is_empty());
    assert_eq!(set.len(), 0);
}

#[test]
fn insert_and_contains() {
    let mut set = CoordSet::new();
    let c = Coord::new(0).unwrap();
    assert!(!set.contains(c));
    assert!(set.insert(c));
    assert!(set.contains(c));
}

#[test]
fn insert_duplicate() {
    let mut set = CoordSet::new();
    let c = Coord::new(0).unwrap();
    assert!(set.insert(c));
    assert!(!set.insert(c));
}

#[test]
fn remove() {
    let mut set = CoordSet::new();
    let c = Coord::new(0).unwrap();
    set.insert(c);
    assert!(set.remove(c));
    assert!(!set.contains(c));
    assert!(!set.remove(c));
}

#[test]
fn len() {
    let mut set = CoordSet::new();
    for i in 0u16..50 {
        set.insert(Coord::new(i).unwrap());
    }
    assert_eq!(set.len(), 50);
}

#[test]
fn clear() {
    let mut set = CoordSet::new();
    set.insert(Coord::new(0).unwrap());
    set.insert(Coord::new(100).unwrap());
    set.clear();
    assert!(set.is_empty());
}

#[test]
fn union_basic() {
    let mut a = CoordSet::new();
    let mut b = CoordSet::new();
    a.insert(Coord::new(0).unwrap());
    b.insert(Coord::new(1).unwrap());
    let u = a.union(&b);
    assert!(u.contains(Coord::new(0).unwrap()));
    assert!(u.contains(Coord::new(1).unwrap()));
}

#[test]
fn intersection_basic() {
    let mut a = CoordSet::new();
    let mut b = CoordSet::new();
    a.insert(Coord::new(0).unwrap());
    a.insert(Coord::new(1).unwrap());
    b.insert(Coord::new(1).unwrap());
    b.insert(Coord::new(2).unwrap());
    let i = a.intersection(&b);
    assert!(!i.contains(Coord::new(0).unwrap()));
    assert!(i.contains(Coord::new(1).unwrap()));
    assert!(!i.contains(Coord::new(2).unwrap()));
}

#[test]
fn difference_basic() {
    let mut a = CoordSet::new();
    let mut b = CoordSet::new();
    a.insert(Coord::new(0).unwrap());
    a.insert(Coord::new(1).unwrap());
    b.insert(Coord::new(1).unwrap());
    let d = a.difference(&b);
    assert!(d.contains(Coord::new(0).unwrap()));
    assert!(!d.contains(Coord::new(1).unwrap()));
}

#[test]
fn symmetric_difference() {
    let mut a = CoordSet::new();
    let mut b = CoordSet::new();
    a.insert(Coord::new(0).unwrap());
    b.insert(Coord::new(1).unwrap());
    let sd = a.symmetric_difference(&b);
    assert!(sd.contains(Coord::new(0).unwrap()));
    assert!(sd.contains(Coord::new(1).unwrap()));
    a.insert(Coord::new(2).unwrap());
    b.insert(Coord::new(2).unwrap());
    let sd2 = a.symmetric_difference(&b);
    assert!(!sd2.contains(Coord::new(2).unwrap()));
}

#[test]
fn subset() {
    let mut a = CoordSet::new();
    let mut b = CoordSet::new();
    a.insert(Coord::new(0).unwrap());
    a.insert(Coord::new(1).unwrap());
    b.insert(Coord::new(0).unwrap());
    b.insert(Coord::new(1).unwrap());
    b.insert(Coord::new(2).unwrap());
    assert!(a.is_subset(&b));
    assert!(!b.is_subset(&a));
}

#[test]
fn superset() {
    let mut a = CoordSet::new();
    let mut b = CoordSet::new();
    a.insert(Coord::new(0).unwrap());
    a.insert(Coord::new(1).unwrap());
    a.insert(Coord::new(2).unwrap());
    b.insert(Coord::new(0).unwrap());
    b.insert(Coord::new(1).unwrap());
    assert!(a.is_superset(&b));
    assert!(!b.is_superset(&a));
}

#[test]
fn disjoint() {
    let mut a = CoordSet::new();
    let mut b = CoordSet::new();
    a.insert(Coord::new(0).unwrap());
    b.insert(Coord::new(1).unwrap());
    assert!(a.is_disjoint(&b));
    b.insert(Coord::new(0).unwrap());
    assert!(!a.is_disjoint(&b));
}

#[test]
fn iter_empty() {
    let set = CoordSet::new();
    assert_eq!(set.iter().count(), 0);
}

#[test]
fn iter_non_empty() {
    let mut set = CoordSet::new();
    set.insert(Coord::new(0).unwrap());
    set.insert(Coord::new(11171).unwrap());
    let v: Vec<_> = set.iter().collect();
    assert_eq!(v.len(), 2);
    assert!(v.contains(&Coord::new(0).unwrap()));
    assert!(v.contains(&Coord::new(11171).unwrap()));
}

#[test]
fn into_iter() {
    let mut set = CoordSet::new();
    set.insert(Coord::new(5).unwrap());
    let v: Vec<_> = (&set).into_iter().collect();
    assert_eq!(v, vec![Coord::new(5).unwrap()]);
}

#[test]
fn from_iterator() {
    let coords: Vec<_> = (0..10u16).map(|i| Coord::new(i).unwrap()).collect();
    let set: CoordSet = coords.into_iter().collect();
    assert_eq!(set.len(), 10);
}

#[test]
fn index_trait() {
    let mut set = CoordSet::new();
    let c = Coord::new(7).unwrap();
    assert!(!set[c]);
    set.insert(c);
    assert!(set[c]);
}

#[test]
fn fill_all() {
    let mut set = CoordSet::new();
    for i in 0u16..11172 {
        set.insert(Coord::new(i).unwrap());
    }
    assert_eq!(set.len(), 11172);
    assert!(!set.is_empty());
    for i in 0u16..11172 {
        assert!(set.contains(Coord::new(i).unwrap()));
    }
}

#[test]
fn remove_all() {
    let mut set = CoordSet::new();
    for i in 0u16..11172 {
        set.insert(Coord::new(i).unwrap());
    }
    for i in 0u16..11172 {
        set.remove(Coord::new(i).unwrap());
    }
    assert!(set.is_empty());
    assert_eq!(set.len(), 0);
}

#[test]
fn display_format() {
    let mut set = CoordSet::new();
    set.insert(Coord::new(0).unwrap());
    let s = format!("{}", set);
    assert!(s.contains("가"));
}

#[test]
fn clone_eq() {
    let mut a = CoordSet::new();
    a.insert(Coord::new(0).unwrap());
    let b = a;
    assert_eq!(a, b);
    assert!(a.contains(Coord::new(0).unwrap()));
}

#[test]
fn default_is_empty() {
    let set: CoordSet = Default::default();
    assert!(set.is_empty());
}

#[test]
fn get_present() {
    let mut set = CoordSet::new();
    let c = Coord::new(42).unwrap();
    set.insert(c);
    assert_eq!(set.get(&c), Some(&c));
}

#[test]
fn get_absent() {
    let set = CoordSet::new();
    assert_eq!(set.get(&Coord::new(0).unwrap()), None);
}

#[test]
fn take_present() {
    let mut set = CoordSet::new();
    let c = Coord::new(42).unwrap();
    set.insert(c);
    assert_eq!(set.take(&c), Some(c));
    assert!(!set.contains(c));
}

#[test]
fn take_absent() {
    let mut set = CoordSet::new();
    assert_eq!(set.take(&Coord::new(0).unwrap()), None);
}

#[test]
fn retain_all() {
    let mut set = CoordSet::new();
    for i in 0u16..10 {
        set.insert(Coord::new(i).unwrap());
    }
    set.retain(|_| true);
    assert_eq!(set.len(), 10);
}

#[test]
fn retain_odd() {
    let mut set = CoordSet::new();
    for i in 0u16..10 {
        set.insert(Coord::new(i).unwrap());
    }
    set.retain(|c| c.index() % 2 == 0);
    assert_eq!(set.len(), 5);
    for i in 0u16..10 {
        let c = Coord::new(i).unwrap();
        assert_eq!(set.contains(c), i % 2 == 0);
    }
}

#[test]
fn retain_empty() {
    let mut set = CoordSet::new();
    set.retain(|_| true);
    assert!(set.is_empty());
}

#[test]
fn capacity_instance() {
    let set = CoordSet::new();
    assert_eq!(set.capacity(), 11172);
}
