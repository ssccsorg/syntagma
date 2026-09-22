//! The quantized matrix-vector product.

use crate::elements::Elements;
use crate::order::Order;
use crate::requant::Requant;

/// Computes `y[i] = sum over j of a[i, j] * x[j]`.
///
/// An operand is read at its coordinate. The coordinate is the address and the
/// backing order only decides where the bytes sit, so the result is the same for
/// any order over the same logical elements.
///
/// The accumulator cannot overflow at any addressable width. The largest term is
/// `128 * 127`, and `128 * 127 * 11_172` is well below `i32::MAX`.
pub fn gemv<const R: usize, const C: usize, O: Order, E: Elements<R, C, O>>(
    a: &E,
    x: &[i8; C],
    y: &mut [i32; R],
) {
    for (i, out) in y.iter_mut().enumerate() {
        let mut accumulator: i32 = 0;
        for (j, activation) in x.iter().enumerate() {
            accumulator += i32::from(a.element(i, j)) * i32::from(*activation);
        }
        *out = accumulator;
    }
}

/// Computes `y[i]` as the rescale of `sum over j of a[i, j] * x[j]`.
///
/// The accumulator bound is the one [`gemv`] states and the rescale is the one
/// [`Requant::requantize`] states, so an element is what those two steps produce in
/// sequence rather than a second rounding of them.
pub fn gemv_requantized<const R: usize, const C: usize, O: Order, E: Elements<R, C, O>>(
    a: &E,
    x: &[i8; C],
    requant: &Requant,
    y: &mut [i8; R],
) {
    for (i, out) in y.iter_mut().enumerate() {
        let mut accumulator: i32 = 0;
        for (j, activation) in x.iter().enumerate() {
            accumulator += i32::from(a.element(i, j)) * i32::from(*activation);
        }
        *out = requant.requantize(accumulator);
    }
}
