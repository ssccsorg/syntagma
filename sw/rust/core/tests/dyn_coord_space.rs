use tagma_core::{Coord, DynCoordSpace};

#[test]
fn empty() {
    let m: DynCoordSpace<u32> = DynCoordSpace::new();
    assert_eq!(m.at(&[Coord::new(0).unwrap()]), None);
}

#[test]
fn depth_1() {
    let mut m = DynCoordSpace::new();
    let c = Coord::new(42).unwrap();
    assert_eq!(m.place(&[c], 7), None);
    assert_eq!(m.at(&[c]), Some(&7));
}

#[test]
fn depth_2() {
    let mut m = DynCoordSpace::new();
    let path = [Coord::new(0).unwrap(), Coord::new(1).unwrap()];
    m.place(&path, 42);
    assert_eq!(m.at(&path), Some(&42));
}

#[test]
fn depth_3() {
    let mut m = DynCoordSpace::new();
    let path = [
        Coord::new(0).unwrap(),
        Coord::new(1).unwrap(),
        Coord::new(2).unwrap(),
    ];
    m.place(&path, 99);
    assert_eq!(m.at(&path), Some(&99));
}

#[test]
fn independent_paths() {
    let mut m = DynCoordSpace::new();
    let a = [Coord::new(0).unwrap(), Coord::new(0).unwrap()];
    let b = [Coord::new(0).unwrap(), Coord::new(1).unwrap()];
    m.place(&a, 10);
    m.place(&b, 20);
    assert_eq!(m.at(&a), Some(&10));
    assert_eq!(m.at(&b), Some(&20));
}

#[test]
fn overwrite() {
    let mut m = DynCoordSpace::new();
    let path = [Coord::new(5).unwrap()];
    m.place(&path, 1);
    assert_eq!(m.place(&path, 2), Some(1));
    assert_eq!(m.at(&path), Some(&2));
}

#[test]
fn vacate() {
    let mut m = DynCoordSpace::new();
    let path = [Coord::new(0).unwrap(), Coord::new(1).unwrap()];
    m.place(&path, 42);
    assert_eq!(m.vacate(&path), Some(42));
    assert_eq!(m.at(&path), None);
}

#[test]
fn mixed_depths() {
    let mut m = DynCoordSpace::new();
    let d1 = [Coord::new(1).unwrap()];
    let d3 = [
        Coord::new(1).unwrap(),
        Coord::new(2).unwrap(),
        Coord::new(3).unwrap(),
    ];
    m.place(&d1, 10);
    m.place(&d3, 30);
    assert_eq!(m.at(&d3), Some(&30));
    assert_eq!(m.at(&d1), Some(&10));
}

#[test]
fn clear() {
    let mut m = DynCoordSpace::new();
    m.place(&[Coord::new(0).unwrap()], 1);
    m.place(&[Coord::new(1).unwrap(), Coord::new(0).unwrap()], 2);
    m.clear();
    assert_eq!(m.at(&[Coord::new(0).unwrap()]), None);
    assert_eq!(
        m.at(&[Coord::new(1).unwrap(), Coord::new(0).unwrap()]),
        None
    );
}

#[test]
fn boundary() {
    let mut m = DynCoordSpace::new();
    let first = Coord::new(0).unwrap();
    let last = Coord::new(11171).unwrap();
    m.place(&[first, last], 42);
    assert_eq!(m.at(&[first, last]), Some(&42));
}

#[test]
fn missing_path() {
    let m: DynCoordSpace<u32> = DynCoordSpace::new();
    assert_eq!(
        m.at(&[Coord::new(0).unwrap(), Coord::new(0).unwrap()]),
        None
    );
}

#[test]
fn empty_path_get_returns_none() {
    let m: DynCoordSpace<u32> = DynCoordSpace::new();
    assert_eq!(m.at(&[]), None);
}

#[test]
fn empty_path_remove_returns_none() {
    let mut m: DynCoordSpace<u32> = DynCoordSpace::new();
    assert_eq!(m.vacate(&[]), None);
}

#[test]
#[should_panic(expected = "path must not be empty")]
fn empty_path_insert_panics() {
    let mut m: DynCoordSpace<u32> = DynCoordSpace::new();
    m.place(&[], 42);
}

#[test]
fn clone_independent() {
    let mut a = DynCoordSpace::new();
    a.place(&[Coord::new(0).unwrap()], 1);
    a.place(&[Coord::new(1).unwrap(), Coord::new(2).unwrap()], 2);
    let mut b = a.clone();
    b.place(&[Coord::new(3).unwrap()], 3);
    assert_eq!(a.entry_count(), 2);
    assert_eq!(b.entry_count(), 3);
}

#[test]
fn iter_yields_all_entries() {
    let mut m = DynCoordSpace::new();
    m.place(&[Coord::new(0).unwrap()], 10);
    m.place(&[Coord::new(1).unwrap(), Coord::new(2).unwrap()], 20);
    let entries: Vec<_> = m.iter().collect();
    assert_eq!(entries.len(), 2);
}

#[test]
fn iter_empty() {
    let m: DynCoordSpace<u32> = DynCoordSpace::new();
    assert_eq!(m.iter().count(), 0);
}
