//! Proofs over the rescale's whole domain.
//!
//! The crate's tests sample the contract: a grid of accumulators, multipliers, and
//! shifts. A grid can show that an implementation is right at the points it holds, and
//! it cannot show that the sixty-four bit intermediate holds the product at every
//! multiplier and every allowed shift, which is the argument the shift bound rests on
//! rather than a remark beside it.
//!
//! The reference here is written in wider arithmetic than the implementation uses, so
//! the proof is that sixty-four bits are enough rather than that two copies of one
//! formula agree.
//!
//! Only `cargo kani` compiles this module.

use crate::requant::Requant;

/// The rescale as this crate documents it, in arithmetic wide enough to be exact.
fn reference(accumulator: i32, multiplier: i32, shift: u32) -> i8 {
    let product = i128::from(accumulator) * i128::from(multiplier);
    let divisor = 1i128 << shift;
    let rounding = divisor / 2;
    let scaled = floor_divide(product + rounding, divisor);

    if scaled > i128::from(i8::MAX) {
        i8::MAX
    } else if scaled < i128::from(i8::MIN) {
        i8::MIN
    } else {
        scaled as i8
    }
}

/// Division that floors, which the language's operator does not do for a negative
/// numerator and an arithmetic shift does.
fn floor_divide(numerator: i128, denominator: i128) -> i128 {
    let quotient = numerator / denominator;
    if numerator % denominator != 0 && (numerator < 0) != (denominator < 0) {
        quotient - 1
    } else {
        quotient
    }
}

/// The implementation's sixty-four bits produce the stated result at every point of
/// the domain, which is what makes the intermediate a width argument rather than a
/// hope.
#[kani::proof]
fn the_rescale_is_the_stated_result_at_every_point() {
    let accumulator: i32 = kani::any();
    let multiplier: i32 = kani::any();
    let shift: u32 = kani::any();

    kani::assume(shift <= Requant::MAX_SHIFT);

    let requant = Requant::new(multiplier, shift);

    assert_eq!(
        requant.requantize(accumulator),
        reference(accumulator, multiplier, shift)
    );
}

/// The product and its rounding constant fit the intermediate at every allowed shift.
///
/// This is the bound `MAX_SHIFT` exists for, and it is stated apart from the equality
/// above because it is the design decision rather than its consequence: the product of
/// two thirty-two bit values occupies at most sixty-two bits, and a constant derived
/// from a shift of sixty-two adds one more.
#[kani::proof]
fn the_intermediate_holds_the_product_at_every_allowed_shift() {
    let accumulator: i32 = kani::any();
    let multiplier: i32 = kani::any();
    let shift: u32 = kani::any();

    kani::assume(shift <= Requant::MAX_SHIFT);

    let product = i64::from(accumulator) * i64::from(multiplier);
    let rounding = if shift == 0 { 0 } else { 1i64 << (shift - 1) };

    assert!(product.checked_add(rounding).is_some());
}
