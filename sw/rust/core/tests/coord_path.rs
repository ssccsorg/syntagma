use tagma_core::{Coord, CoordPath};

#[test]
fn path_from_coord() {
    let c = Coord::new(42).unwrap();
    let path = CoordPath::<1>::new([c]);
    assert_eq!(path.coords()[0], c);
    assert_eq!(path.len(), 1);
}

#[test]
fn path_from_array() {
    let a = Coord::new(0).unwrap();
    let b = Coord::new(1).unwrap();
    let path = CoordPath::<2>::new([a, b]);
    assert_eq!(path.coords()[0], a);
    assert_eq!(path.coords()[1], b);
}

#[test]
fn path_into_conversion() {
    let c = Coord::new(7).unwrap();
    let path: CoordPath<1> = c.into();
    assert_eq!(path.coords()[0], c);
}

#[test]
fn path_display() {
    let c = Coord::new(0).unwrap();
    let path = CoordPath::<1>::new([c]);
    let s = format!("{}", path);
    assert!(s.contains("가"));
}

#[test]
fn path_eq() {
    let a = CoordPath::<2>::new([Coord::new(0).unwrap(), Coord::new(1).unwrap()]);
    let b = CoordPath::<2>::new([Coord::new(0).unwrap(), Coord::new(1).unwrap()]);
    assert_eq!(a, b);
}

#[test]
fn path_ne() {
    let a = CoordPath::<2>::new([Coord::new(0).unwrap(), Coord::new(1).unwrap()]);
    let b = CoordPath::<2>::new([Coord::new(0).unwrap(), Coord::new(2).unwrap()]);
    assert_ne!(a, b);
}
