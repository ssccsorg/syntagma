use tagma_core::{Coord, CoordCube, CoordPath};

// ---------------------------------------------------------------------------
// Construction
// ---------------------------------------------------------------------------

#[test]
fn cube_from_path() {
    let path = CoordPath::<6>::new([
        Coord::new(0).unwrap(),
        Coord::new(1).unwrap(),
        Coord::new(2).unwrap(),
        Coord::new(3).unwrap(),
        Coord::new(4).unwrap(),
        Coord::new(5).unwrap(),
    ]);
    let cube = CoordCube::<6, 3, 2>::from_path(path);
    assert_eq!(cube.ndim(), 3);
    assert_eq!(cube.resolution(), 2);
    assert_eq!(cube.total_characters(), 6);
}

#[test]
fn cube_into_path_roundtrip() {
    let path = CoordPath::<4>::new([
        Coord::new(10).unwrap(),
        Coord::new(20).unwrap(),
        Coord::new(30).unwrap(),
        Coord::new(40).unwrap(),
    ]);
    let cube = CoordCube::<4, 2, 2>::from_path(path);
    let path_back: CoordPath<4> = cube.into_path();
    assert_eq!(path_back.coords()[0].index(), 10);
    assert_eq!(path_back.coords()[3].index(), 40);
}

#[test]
fn cube_from_path_via_from_trait() {
    let path = CoordPath::<2>::new([Coord::new(42).unwrap(), Coord::new(99).unwrap()]);
    let cube: CoordCube<2, 2, 1> = path.into();
    assert_eq!(cube.axis(0).coords()[0].index(), 42);
    assert_eq!(cube.axis(1).coords()[0].index(), 99);
}

#[test]
fn cube_into_path_via_from_trait() {
    let path = CoordPath::<2>::new([Coord::new(7).unwrap(), Coord::new(8).unwrap()]);
    let cube = CoordCube::<2, 2, 1>::from_path(path);
    let path_back: CoordPath<2> = cube.into();
    assert_eq!(path_back.coords()[0].index(), 7);
}

#[test]
fn cube_as_path() {
    let path = CoordPath::<2>::new([Coord::new(10).unwrap(), Coord::new(20).unwrap()]);
    let cube = CoordCube::<2, 2, 1>::from_path(path);
    let path_ref = cube.as_path();
    assert_eq!(path_ref.coords()[0].index(), 10);
    assert_eq!(path_ref.coords()[1].index(), 20);
}

#[test]
fn cube_coords_accessor() {
    let path = CoordPath::<2>::new([Coord::new(10).unwrap(), Coord::new(20).unwrap()]);
    let cube = CoordCube::<2, 2, 1>::from_path(path);
    let coords = cube.coords();
    assert_eq!(coords.len(), 2);
    assert_eq!(coords[0].index(), 10);
    assert_eq!(coords[1].index(), 20);
}

#[test]
fn cube_from_path_unchecked() {
    let path = CoordPath::<2>::new([Coord::new(7).unwrap(), Coord::new(8).unwrap()]);
    // SAFETY: D * R = 2 * 1 = 2 = N
    let cube = unsafe { CoordCube::<2, 2, 1>::from_path_unchecked(path) };
    assert_eq!(cube.axis(0).coords()[0].index(), 7);
    assert_eq!(cube.axis(1).coords()[0].index(), 8);
}

#[test]
#[should_panic(expected = "N=4 must equal D*R")]
fn cube_invalid_dimensions_panics() {
    let path = CoordPath::<4>::new([
        Coord::new(0).unwrap(),
        Coord::new(0).unwrap(),
        Coord::new(0).unwrap(),
        Coord::new(0).unwrap(),
    ]);
    let _cube = CoordCube::<4, 3, 1>::from_path(path);
}

// ---------------------------------------------------------------------------
// Axis access
// ---------------------------------------------------------------------------

#[test]
fn cube_axis_single_character() {
    let path = CoordPath::<3>::new([
        Coord::new(111).unwrap(),
        Coord::new(222).unwrap(),
        Coord::new(333).unwrap(),
    ]);
    let cube = CoordCube::<3, 3, 1>::from_path(path);
    assert_eq!(cube.axis(0).coords()[0].index(), 111);
    assert_eq!(cube.axis(1).coords()[0].index(), 222);
    assert_eq!(cube.axis(2).coords()[0].index(), 333);
}

#[test]
fn cube_axis_multi_character() {
    let path = CoordPath::<6>::new([
        Coord::new(0).unwrap(),
        Coord::new(1).unwrap(),
        Coord::new(2).unwrap(),
        Coord::new(3).unwrap(),
        Coord::new(4).unwrap(),
        Coord::new(5).unwrap(),
    ]);
    let cube = CoordCube::<6, 3, 2>::from_path(path);
    let axis0 = cube.axis(0);
    assert_eq!(axis0.coords()[0].index(), 0);
    assert_eq!(axis0.coords()[1].index(), 1);
    let axis1 = cube.axis(1);
    assert_eq!(axis1.coords()[0].index(), 2);
    assert_eq!(axis1.coords()[1].index(), 3);
    let axis2 = cube.axis(2);
    assert_eq!(axis2.coords()[0].index(), 4);
    assert_eq!(axis2.coords()[1].index(), 5);
}

#[test]
#[should_panic]
fn cube_axis_out_of_range() {
    let path = CoordPath::<2>::new([Coord::new(0).unwrap(), Coord::new(0).unwrap()]);
    let cube = CoordCube::<2, 2, 1>::from_path(path);
    let _ = cube.axis(2);
}

// ---------------------------------------------------------------------------
// coord_at
// ---------------------------------------------------------------------------

#[test]
fn cube_coord_at() {
    let path = CoordPath::<4>::new([
        Coord::new(10).unwrap(),
        Coord::new(20).unwrap(),
        Coord::new(30).unwrap(),
        Coord::new(40).unwrap(),
    ]);
    let cube = CoordCube::<4, 2, 2>::from_path(path);
    assert_eq!(cube.coord_at(0, 0).index(), 10);
    assert_eq!(cube.coord_at(0, 1).index(), 20);
    assert_eq!(cube.coord_at(1, 0).index(), 30);
    assert_eq!(cube.coord_at(1, 1).index(), 40);
}

// ---------------------------------------------------------------------------
// Display
// ---------------------------------------------------------------------------

#[test]
fn cube_display() {
    let path = CoordPath::<2>::new([Coord::new(0).unwrap(), Coord::new(0).unwrap()]);
    let cube = CoordCube::<2, 2, 1>::from_path(path);
    let s = format!("{}", cube);
    assert!(s.contains("CoordCube"));
    assert!(s.contains("2, 2, 1"));
}

// ---------------------------------------------------------------------------
// Equality
// ---------------------------------------------------------------------------

#[test]
fn cube_eq() {
    let a = CoordCube::<2, 2, 1>::from_path(CoordPath::new([
        Coord::new(1).unwrap(),
        Coord::new(2).unwrap(),
    ]));
    let b = CoordCube::<2, 2, 1>::from_path(CoordPath::new([
        Coord::new(1).unwrap(),
        Coord::new(2).unwrap(),
    ]));
    assert_eq!(a, b);
}

#[test]
fn cube_ne() {
    let a = CoordCube::<2, 2, 1>::from_path(CoordPath::new([
        Coord::new(1).unwrap(),
        Coord::new(2).unwrap(),
    ]));
    let b = CoordCube::<2, 2, 1>::from_path(CoordPath::new([
        Coord::new(1).unwrap(),
        Coord::new(3).unwrap(),
    ]));
    assert_ne!(a, b);
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

#[test]
fn cube_single_dimension() {
    let path = CoordPath::<3>::new([
        Coord::new(5).unwrap(),
        Coord::new(10).unwrap(),
        Coord::new(15).unwrap(),
    ]);
    let cube = CoordCube::<3, 1, 3>::from_path(path);
    assert_eq!(cube.ndim(), 1);
    assert_eq!(cube.resolution(), 3);
    let axis = cube.axis(0);
    assert_eq!(axis.coords()[0].index(), 5);
    assert_eq!(axis.coords()[1].index(), 10);
    assert_eq!(axis.coords()[2].index(), 15);
}

#[test]
fn cube_single_character_per_dim() {
    let path = CoordPath::<5>::new([
        Coord::new(0).unwrap(),
        Coord::new(1).unwrap(),
        Coord::new(2).unwrap(),
        Coord::new(3).unwrap(),
        Coord::new(4).unwrap(),
    ]);
    let cube = CoordCube::<5, 5, 1>::from_path(path);
    assert_eq!(cube.ndim(), 5);
    for i in 0..5 {
        assert_eq!(cube.axis(i).coords()[0].index(), i as u16);
    }
}
