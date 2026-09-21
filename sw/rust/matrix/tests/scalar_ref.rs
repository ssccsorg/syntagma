//! The shipped product against a scalar reference, and the coordinate against the
//! order-resolved index.

mod common;

use tagma_core::{Coord, CoordPath};
use tagma_matrix::{gemv, Elements, Matrix};

#[test]
fn the_product_matches_the_scalar_reference_bit_for_bit() {
    const R: usize = 7;
    const C: usize = 5;

    let a = common::row_major::<R, C>();
    let x = common::activation::<C>();

    let mut y = [0i32; R];
    gemv(&a, &x, &mut y);

    assert_eq!(y, common::reference(&a, &x));
}

#[test]
fn a_coordinate_reads_the_value_the_order_resolved_index_reads() {
    const R: usize = 7;
    const C: usize = 5;

    let a = common::row_major::<R, C>();

    for i in 0..R {
        for j in 0..C {
            let path = a.path_of(i, j).expect("in range");
            assert_eq!(a.at(path), Some(common::value_at(i, j)));
            assert_eq!(a.at(path), Some(a.get(i, j)));
        }
    }
}

#[test]
fn a_position_outside_the_matrix_has_no_coordinate() {
    const R: usize = 7;
    const C: usize = 5;

    let a = common::row_major::<R, C>();

    assert_eq!(a.path_of(R, 0), None);
    assert_eq!(a.path_of(0, C), None);

    let row = Coord::new(R as u16).expect("in range");
    let column = Coord::new(0).expect("in range");
    assert_eq!(a.at(CoordPath::new([row, column])), None);
}

#[test]
fn a_matrix_can_be_written_through_its_index_surface() {
    const R: usize = 3;
    const C: usize = 2;

    let mut a = Matrix::<R, C>::new([[0i8; C]; R]);
    for i in 0..R {
        for j in 0..C {
            a.set(i, j, common::value_at(i, j));
        }
    }

    for i in 0..R {
        for j in 0..C {
            assert_eq!(a.get(i, j), common::value_at(i, j));
        }
    }
}

/// The type is `Copy`, as the family's other fixed-size types are, so a copy is a
/// second buffer rather than a second name for one.
#[test]
fn a_copy_is_independent_of_its_source() {
    let mut a = common::row_major::<2, 3>();
    let mut b = a;

    assert_eq!(a, b);

    a.set(0, 0, 99);
    b.set(1, 2, -99);

    assert_eq!(a.get(0, 0), 99);
    assert_eq!(b.get(0, 0), common::value_at(0, 0));
    assert_eq!(b.get(1, 2), -99);
    assert_eq!(a.get(1, 2), common::value_at(1, 2));
}
