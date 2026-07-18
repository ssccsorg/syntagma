use tagma_core::Coord;

#[test]
fn all_11172_coords_are_valid() {
    for i in 0..11172u16 {
        assert!(Coord::new(i).is_some());
    }
    assert!(Coord::new(11172).is_none());
}

#[test]
fn roundtrip_axes() {
    for i in 0..19 {
        for m in 0..21 {
            for f in 0..28 {
                let c = Coord::from_axes(i, m, f).unwrap();
                assert_eq!((i, m, f), c.to_axes());
            }
        }
    }
}

#[test]
fn roundtrip_code_point() {
    for raw in [0u16, 1, 11171, 4444, 8888] {
        let c = Coord::new(raw).unwrap();
        let cp = c.to_code_point();
        let back = Coord::from_code_point(cp).unwrap();
        assert_eq!(c, back);
    }
}

#[test]
fn char_roundtrip() {
    let c = Coord::from_axes(0, 0, 0).unwrap();
    assert_eq!(c.to_char(), '가');
    #[cfg(feature = "alloc")]
    assert_eq!(c.to_hangul_string(), "가");

    let last = Coord::new(11171).unwrap();
    assert_eq!(last.to_char(), '힣');
}

#[test]
fn hamming_distance_same() {
    let a = Coord::new(0).unwrap();
    assert_eq!(a.hamming_distance(a), (0, 0, 0));
}

#[test]
fn hamming_distance_different() {
    let a = Coord::from_axes(0, 0, 0).unwrap();
    let b = Coord::from_axes(3, 5, 7).unwrap();
    assert_eq!(a.hamming_distance(b), (3, 5, 7));
}

#[test]
fn coordinate_formula_smoke() {
    let ga = Coord::from_char('가').unwrap();
    assert_eq!(ga.to_axes(), (0, 0, 0));

    let hih = Coord::from_char('힣').unwrap();
    assert_eq!(hih.to_axes(), (18, 20, 27));
    assert_eq!(hih.index(), 11171);
}

#[test]
fn from_axes_rejects_oob() {
    assert!(Coord::from_axes(19, 0, 0).is_none());
    assert!(Coord::from_axes(0, 21, 0).is_none());
    assert!(Coord::from_axes(0, 0, 28).is_none());
}

#[test]
fn hamming_distance_max() {
    let a = Coord::from_axes(0, 0, 0).unwrap();
    let b = Coord::from_axes(18, 20, 27).unwrap();
    assert_eq!(a.hamming_distance(b), (18, 20, 27));
}

#[test]
fn filler_positions_invalid() {
    for cp in 0xD7A4u16..=0xD7AF {
        assert!(Coord::from_code_point(cp).is_none());
    }
}

#[test]
fn code_point_outside_block() {
    assert!(Coord::from_code_point(0x0041).is_none());
    assert!(Coord::from_code_point(0xAC00 - 1).is_none());
    assert!(Coord::from_code_point(0xD7AF + 1).is_none());
}

#[test]
fn serialization_roundtrip() {
    for raw in [0u16, 1, 256, 11171, 5555] {
        let c = Coord::new(raw).unwrap();
        assert_eq!(Coord::from_le_bytes(c.to_le_bytes()), Some(c));
        assert_eq!(Coord::from_be_bytes(c.to_be_bytes()), Some(c));
    }
}
