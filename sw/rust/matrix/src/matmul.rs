//! The product of two matrices.

use crate::elements::Elements;
use crate::matrix::Matrix;
use crate::order::Order;
use crate::requant::Requant;

/// Computes `out[i, j] = sum over k of a[i, k] * b[k, j]`.
///
/// Both operands are read at their coordinates, so the physical order of either
/// backing buffer is not part of the result and the same logical operands produce
/// the same product wherever their bytes sit.
///
/// The accumulator cannot overflow at any addressable width. The largest term is
/// `128 * 127` and a contraction is at most `Coord::N_VALID` terms long, so the
/// largest magnitude is `128 * 127 * 11_172`, well below `i32::MAX`.
pub fn matmul<
    const R: usize,
    const K: usize,
    const C: usize,
    OA: Order,
    OB: Order,
    A: Elements<R, K, OA>,
    B: Elements<K, C, OB>,
>(
    a: &A,
    b: &B,
    out: &mut [[i32; C]; R],
) {
    for (i, row) in out.iter_mut().enumerate() {
        for (j, element) in row.iter_mut().enumerate() {
            let mut accumulator: i32 = 0;
            for k in 0..K {
                accumulator += i32::from(a.element(i, k)) * i32::from(b.element(k, j));
            }
            *element = accumulator;
        }
    }
}

/// Computes the product and the rescale of it, with no buffer for the accumulators.
///
/// The accumulator is rescaled as soon as it is complete, so the product never
/// occupies memory of its own and a layer costs one element of scratch rather than a
/// matrix of it.
pub fn matmul_requantized<
    const R: usize,
    const K: usize,
    const C: usize,
    OA: Order,
    OB: Order,
    OM: Order,
    A: Elements<R, K, OA>,
    B: Elements<K, C, OB>,
>(
    a: &A,
    b: &B,
    requant: &Requant,
    out: &mut Matrix<R, C, OM>,
) {
    for i in 0..R {
        for j in 0..C {
            let mut accumulator: i32 = 0;
            for k in 0..K {
                accumulator += i32::from(a.element(i, k)) * i32::from(b.element(k, j));
            }
            out.write(i, j, requant.requantize(accumulator));
        }
    }
}
