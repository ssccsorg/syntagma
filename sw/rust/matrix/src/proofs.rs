//! Proofs over the two surfaces where something from outside meets this crate.
//!
//! The rescale's domain is every accumulator, multiplier, and shift a model can carry.
//! The reader's domain is every byte slice a device can send. The crate's tests sample
//! both, and a sample can show that an implementation is right at the points it holds
//! and cannot show what happens at the points nobody tried.
//!
//! The reference for the rescale is written in wider arithmetic than the implementation
//! uses, so the proof is that sixty-four bits are enough rather than that two copies of
//! one formula agree.
//!
//! Only `cargo kani` compiles this module.

use crate::elements::Elements;
use crate::matrix::Matrix;
use crate::order::RowMajor;
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

const R: usize = 2;
const C: usize = 3;

/// The bytes a two by three matrix occupies, and room past them, so that a stream the
/// shape does not account for is inside the harness's domain as well.
const LIMIT: usize = 9 + R * C * 5 + 8;

type Target = Matrix<R, C, RowMajor>;

/// A slice of a length the caller chooses, which is what a device sends.
fn stream() -> ([u8; LIMIT], usize) {
    let bytes: [u8; LIMIT] = kani::any();
    let length: usize = kani::any();

    kani::assume(length <= LIMIT);

    (bytes, length)
}

/// The reader does not panic on any stream, of any content and any length.
///
/// Every read it performs is behind a length check, and the check is the whole of the
/// safety argument: a device under a fault can send anything, and a panic on the target
/// is not a refusal but a stop.
#[kani::proof]
fn the_reader_does_not_panic_on_any_stream() {
    let (bytes, length) = stream();

    let _ = Target::decode(&bytes[..length]);
}

/// A stream the reader accepts writes back as the bytes it arrived in.
///
/// This is the stronger half of the reader's contract. It rejects a record whose
/// coordinate names another element than its own position, and a length the shape does
/// not account for, so what it accepts is exactly what the writer produces: the format
/// has one encoding of a matrix rather than many, and a receiver that agrees on the
/// values has agreed on the bytes.
#[kani::proof]
fn a_stream_the_reader_accepts_writes_back_as_it_arrived() {
    let (bytes, length) = stream();
    let source = &bytes[..length];

    let Ok(matrix) = Target::decode(source) else {
        return;
    };

    let mut out = [0u8; LIMIT];
    let written = matrix.encode_into(&mut out);

    assert_eq!(written, Ok(9 + R * C * 5));
    assert_eq!(&out[..9 + R * C * 5], source);
}
