#pragma once

// tagma_matrix: the quantized matrix-vector product. Mirrors
// sw/rust/matrix/src/gemv.rs.

#include "tagma_matrix/elements.h"
#include "tagma_matrix/requant.h"

#include <array>
#include <cstddef>
#include <cstdint>

namespace tagma_matrix {

// Computes `y[i] = sum over j of a[i, j] * x[j]`.
//
// An operand is read at its coordinate. The coordinate is the address and the
// backing order only decides where the bytes sit, so the result is the same for
// any order over the same logical elements.
//
// The accumulator cannot overflow at any addressable width. The largest term is
// 128 * 128, which is the product of two minimum elements, and
// 128 * 128 * 11172 is well below INT32_MAX.
template <std::size_t R, std::size_t C, class O, class E>
void gemv(const Elements<E, R, C, O>& a, const std::array<int8_t, C>& x,
          std::array<int32_t, R>& y) {
  for (std::size_t i = 0; i < R; ++i) {
    int32_t accumulator = 0;
    for (std::size_t j = 0; j < C; ++j) {
      accumulator += static_cast<int32_t>(a.element(i, j)) *
                     static_cast<int32_t>(x[j]);
    }
    y[i] = accumulator;
  }
}

// Computes `y[i]` as the rescale of `sum over j of a[i, j] * x[j]`.
//
// The accumulator bound is the one `gemv` states and the rescale is the one
// `Requant::requantize` states, so an element is what those two steps produce
// in sequence rather than a second rounding of them.
template <std::size_t R, std::size_t C, class O, class E>
void gemv_requantized(const Elements<E, R, C, O>& a,
                      const std::array<int8_t, C>& x, const Requant& requant,
                      std::array<int8_t, R>& y) {
  for (std::size_t i = 0; i < R; ++i) {
    int32_t accumulator = 0;
    for (std::size_t j = 0; j < C; ++j) {
      accumulator += static_cast<int32_t>(a.element(i, j)) *
                     static_cast<int32_t>(x[j]);
    }
    y[i] = requant.requantize(accumulator);
  }
}

}  // namespace tagma_matrix
