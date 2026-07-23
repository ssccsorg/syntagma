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
    assert_eq!(cube.total_syllables(), 6);
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
#[should_panic(expected = "N=4 must equal D*R")]
fn cube_invalid_dimensions_panics() {
    let path = CoordPath::<4>::new([
        Coord::new(0).unwrap(),
        Coord::new(0).unwrap(),
        Coord::new(0).unwrap(),
        Coord::new(0).unwrap(),
    ]);
    // D * R = 3 * 1 = 3, but N = 4
    let _cube = CoordCube::<4, 3, 1>::from_path(path);
}

// ---------------------------------------------------------------------------
// Axis access
// ---------------------------------------------------------------------------

#[test]
fn cube_axis_single_syllable() {
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
fn cube_axis_multi_syllable() {
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
    let _ = cube.axis(2); // D=2, so indices 0,1 only
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
// Hamming distance
// ---------------------------------------------------------------------------

#[test]
fn hamming_distance_identical() {
    let path = CoordPath::<3>::new([
        Coord::new(100).unwrap(),
        Coord::new(200).unwrap(),
        Coord::new(300).unwrap(),
    ]);
    let a = CoordCube::<3, 3, 1>::from_path(path);
    let b = CoordCube::<3, 3, 1>::from_path(path);
    assert_eq!(a.hamming_distance(&b), 0);
}

#[test]
fn hamming_distance_all_differ() {
    let a = CoordCube::<3, 3, 1>::from_path(CoordPath::new([
        Coord::new(0).unwrap(),
        Coord::new(0).unwrap(),
        Coord::new(0).unwrap(),
    ]));
    let b = CoordCube::<3, 3, 1>::from_path(CoordPath::new([
        Coord::new(1).unwrap(),
        Coord::new(2).unwrap(),
        Coord::new(3).unwrap(),
    ]));
    assert_eq!(a.hamming_distance(&b), 3);
}

#[test]
fn hamming_distance_some_differ() {
    let a = CoordCube::<4, 2, 2>::from_path(CoordPath::new([
        Coord::new(0).unwrap(),
        Coord::new(0).unwrap(),
        Coord::new(0).unwrap(),
        Coord::new(0).unwrap(),
    ]));
    let b = CoordCube::<4, 2, 2>::from_path(CoordPath::new([
        Coord::new(0).unwrap(),
        Coord::new(1).unwrap(),
        Coord::new(2).unwrap(),
        Coord::new(0).unwrap(),
    ]));
    assert_eq!(a.hamming_distance(&b), 2);
}

// ---------------------------------------------------------------------------
// Axis-wise Hamming distance
// ---------------------------------------------------------------------------

#[test]
fn hamming_distance_axes_identical() {
    let path = CoordPath::<4>::new([
        Coord::new(0).unwrap(),
        Coord::new(0).unwrap(),
        Coord::new(0).unwrap(),
        Coord::new(0).unwrap(),
    ]);
    let a = CoordCube::<4, 2, 2>::from_path(path);
    let b = CoordCube::<4, 2, 2>::from_path(path);
    let mut out = [0usize; 2];
    a.hamming_distance_axes(&b, &mut out);
    assert_eq!(out, [0, 0]);
}

#[test]
fn hamming_distance_axes_first_dim() {
    let a = CoordCube::<4, 2, 2>::from_path(CoordPath::new([
        Coord::new(0).unwrap(),
        Coord::new(0).unwrap(),
        Coord::new(0).unwrap(),
        Coord::new(0).unwrap(),
    ]));
    let b = CoordCube::<4, 2, 2>::from_path(CoordPath::new([
        Coord::new(0).unwrap(),
        Coord::new(1).unwrap(),
        Coord::new(0).unwrap(),
        Coord::new(0).unwrap(),
    ]));
    let mut out = [0usize; 2];
    a.hamming_distance_axes(&b, &mut out);
    assert_eq!(out[0], 1);
    assert_eq!(out[1], 0);
}

// ---------------------------------------------------------------------------
// Euclidean distance
// ---------------------------------------------------------------------------

#[test]
fn euclidean_distance_identical() {
    let path = CoordPath::<2>::new([Coord::new(5000).unwrap(), Coord::new(5000).unwrap()]);
    let a = CoordCube::<2, 2, 1>::from_path(path);
    let b = CoordCube::<2, 2, 1>::from_path(path);
    assert!((a.euclidean_distance_approx(&b)).abs() < 1e-10);
}

#[test]
fn euclidean_distance_max_in_one_dim() {
    let a = CoordCube::<2, 2, 1>::from_path(CoordPath::new([
        Coord::new(0).unwrap(),
        Coord::new(0).unwrap(),
    ]));
    let b = CoordCube::<2, 2, 1>::from_path(CoordPath::new([
        Coord::new(11171).unwrap(),
        Coord::new(0).unwrap(),
    ]));
    let d = a.euclidean_distance_approx(&b);
    // Max distance in one dimension = 1.0 (normalised)
    // Since only one dim differs, sqrt(1.0) = 1.0
    assert!((d - 1.0).abs() < 0.001, "got {}", d);
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
    // D=1, R=3: single dimension with 3 syllables
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
fn cube_single_syllable_per_dim() {
    // D=5, R=1: five dimensions, one syllable each
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
