//! The shipped product against a scalar reference, and the coordinate against the
//! order-resolved index.

mod common;

use tagma_matrix::{gemv, Coord, CoordPath, Elements, Matrix, Order};

/// The product written as the plainest possible loop, reading every element
/// through its coordinate rather than through the order-resolved index, so the
/// two access paths are checked against each other.
fn reference<const R: usize, const C: usize, O: Order>(
    a: &Matrix<R, C, O>,
    x: &[i8; C],
) -> [i32; R] {
    let mut y = [0i32; R];
    for (i, out) in y.iter_mut().enumerate() {
        let mut accumulator: i32 = 0;
        for (j, activation) in x.iter().enumerate() {
            let path = match a.path_of(i, j) {
                Some(path) => path,
                None => panic!("reference: position {i}, {j} is outside the matrix"),
            };
            let weight = match a.at(path) {
                Some(weight) => weight,
                None => panic!("reference: coordinate {path} names no element"),
            };
            accumulator += i32::from(weight) * i32::from(*activation);
        }
        *out = accumulator;
    }
    y
}

#[test]
fn the_product_matches_the_scalar_reference_bit_for_bit() {
    const R: usize = 7;
    const C: usize = 5;

    let a = common::row_major::<R, C>();
    let x = common::activation::<C>();

    let mut y = [0i32; R];
    gemv(&a, &x, &mut y);

    assert_eq!(y, reference(&a, &x));
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
