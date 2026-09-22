//! The rescale's contract.
//!
//! Three things have to hold for a quantized model's arithmetic to be reproducible:
//! the intermediate is wider than thirty-two bits, the rounding moves half toward
//! positive infinity rather than away from zero, and a result beyond the element
//! range lands on its end rather than wrapping. Each has a test here, and the whole
//! surface is also checked against a reference written as a floor division, so the
//! direction of the rounding is under test rather than only the width.

mod common;

use tagma_matrix::{requantize_into, Requant};

/// Accumulators that cover zero, both signs, the half cases, both ends of the signed
/// thirty-two bit range, and the largest magnitude a contraction over the coordinate
/// space can reach.
const ACCUMULATORS: [i32; 16] = [
    i32::MIN,
    i32::MIN + 1,
    -181_612_032,
    -1_000_000,
    -300,
    -3,
    -2,
    -1,
    0,
    1,
    2,
    3,
    300,
    1_000_000,
    181_612_032,
    i32::MAX,
];

/// Multipliers that cover zero, the identity, both signs, and both ends of the range.
const MULTIPLIERS: [i32; 8] = [
    0,
    1,
    -1,
    14_508_993,
    38_997_123,
    2_147_483,
    i32::MIN,
    i32::MAX,
];

/// Shifts that cover no shift, both rounding halves that a narrow width cannot see,
/// the ordinary model range, and the bound.
const SHIFTS: [u32; 12] = [0, 1, 2, 7, 15, 16, 27, 30, 31, 32, 47, Requant::MAX_SHIFT];

#[test]
fn the_rescale_agrees_with_a_floor_division_reference() {
    for multiplier in MULTIPLIERS {
        for shift in SHIFTS {
            let rescale = Requant::new(multiplier, shift);
            for accumulator in ACCUMULATORS {
                assert_eq!(
                    rescale.requantize(accumulator),
                    common::requant_reference(accumulator, &rescale),
                    "accumulator {accumulator}, multiplier {multiplier}, shift {shift}"
                );
            }
        }
    }
}

/// The product of a thousand by the multiplier of a real model is fourteen billion,
/// which a thirty-two bit intermediate cannot hold. A build that truncated it would
/// still pass a comparison of small values, so this fixes one case where the low bits
/// decide the answer.
#[test]
fn a_product_beyond_thirty_two_bits_keeps_its_low_bits() {
    let rescale = Requant::new(14_508_993, 30);
    assert_eq!(rescale.requantize(1_000), 14);
    assert_eq!(rescale.requantize(-1_000), -14);
}

/// Half of an odd negative is exactly between two integers, and where it lands is the
/// whole difference between rounding half up and rounding away from zero.
#[test]
fn half_moves_toward_positive_infinity() {
    let rescale = Requant::new(1, 1);

    assert_eq!(rescale.requantize(1), 1);
    assert_eq!(rescale.requantize(3), 2);
    assert_eq!(rescale.requantize(-1), 0);
    assert_eq!(rescale.requantize(-3), -1);
}

#[test]
fn a_shift_of_zero_takes_the_product_as_it_stands() {
    let rescale = Requant::new(3, 0);
    assert_eq!(rescale.requantize(10), 30);
    assert_eq!(rescale.requantize(-10), -30);
}

#[test]
fn a_product_beyond_the_element_range_lands_on_its_end() {
    let wide = Requant::new(i32::MAX, 0);

    assert_eq!(wide.requantize(i32::MAX), i8::MAX);
    assert_eq!(wide.requantize(i32::MIN), i8::MIN);

    let narrow = Requant::new(1_000_000, 0);
    assert_eq!(narrow.requantize(1_000_000), i8::MAX);
    assert_eq!(narrow.requantize(-1_000_000), i8::MIN);
}

/// The product of the two extremes fills sixty-two bits and the rounding constant at
/// the largest shift fills sixty-one, so this is the widest the intermediate ever
/// gets. The result is one rather than two, which is what says the bound is the shift
/// that is allowed rather than one past it.
#[test]
fn the_largest_shift_keeps_the_product_inside_the_intermediate() {
    let rescale = Requant::new(i32::MIN, Requant::MAX_SHIFT);
    assert_eq!(rescale.requantize(i32::MIN), 1);
}

#[test]
#[should_panic(expected = "shift exceeds the sixty-four bit intermediate")]
fn a_shift_beyond_the_intermediate_is_refused() {
    let _ = Requant::new(1, Requant::MAX_SHIFT + 1);
}

#[test]
fn a_run_matches_the_rescale_of_each_accumulator() {
    let rescale = common::requant();
    let accumulators: [i32; 64] = core::array::from_fn(|i| (i as i32 - 32) * 5_791);
    let mut elements = [0i8; 64];

    requantize_into(&accumulators, &rescale, &mut elements);

    for (accumulator, element) in accumulators.iter().zip(elements.iter()) {
        assert_eq!(*element, rescale.requantize(*accumulator));
    }
}

#[test]
#[should_panic(expected = "the destination is shorter than the accumulators")]
fn a_destination_shorter_than_the_accumulators_is_refused() {
    let accumulators = [1i32; 4];
    let mut elements = [0i8; 3];
    requantize_into(&accumulators, &common::requant(), &mut elements);
}

#[test]
fn a_rescale_above_one_scales_up() {
    let rescale = common::requant_above_one();
    assert_eq!(rescale.requantize(50), 102);
    assert_eq!(rescale.requantize(-50), -102);
}
