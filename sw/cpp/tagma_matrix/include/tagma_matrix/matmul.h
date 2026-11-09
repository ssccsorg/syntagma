#pragma once

// tagma_matrix: the product of two matrices. Mirrors
// sw/rust/matrix/src/matmul.rs.

#include "tagma_matrix/elements.h"
#include "tagma_matrix/matrix.h"
#include "tagma_matrix/requant.h"

#include <array>
#include <cstddef>
#include <cstdint>

namespace tagma_matrix {

// Computes `out[i, j] = sum over k of a[i, k] * b[k, j]`.
//
// Both operands are read at their coordinates, so the physical order of either
// backing buffer is not part of the result and the same logical operands
// produce the same product wherever their bytes sit.
//
// The accumulator cannot overflow at any addressable width. The largest term is
// 128 * 128, which is the product of two minimum elements, and a contraction is
// at most `tagma::Coord::kNValid` terms long, so the largest magnitude is
// 128 * 128 * 11172, well below INT32_MAX.
template <std::size_t R, std::size_t K, std::size_t C, class OA, class OB,
          class EA, class EB>
void matmul(const Elements<EA, R, K, OA>& a, const Elements<EB, K, C, OB>& b,
            std::array<std::array<int32_t, C>, R>& out) {
  for (std::size_t i = 0; i < R; ++i) {
    for (std::size_t j = 0; j < C; ++j) {
      int32_t accumulator = 0;
      for (std::size_t k = 0; k < K; ++k) {
        accumulator += static_cast<int32_t>(a.element(i, k)) *
                       static_cast<int32_t>(b.element(k, j));
      }
      out[i][j] = accumulator;
    }
  }
}

// Computes the product and the rescale of it, with no buffer for the
// accumulators.
//
// The accumulator is rescaled as soon as it is complete, so the product never
// occupies memory of its own and a layer costs one element of scratch rather
// than a matrix of it.
template <std::size_t R, std::size_t K, std::size_t C, class OA, class OB,
          class OM, class EA, class EB>
void matmul_requantized(const Elements<EA, R, K, OA>& a,
                        const Elements<EB, K, C, OB>& b, const Requant& requant,
                        Matrix<R, C, OM>& out) {
  for (std::size_t i = 0; i < R; ++i) {
    for (std::size_t j = 0; j < C; ++j) {
      int32_t accumulator = 0;
      for (std::size_t k = 0; k < K; ++k) {
        accumulator += static_cast<int32_t>(a.element(i, k)) *
                       static_cast<int32_t>(b.element(k, j));
      }
      out.write(i, j, requant.requantize(accumulator));
    }
  }
}

}  // namespace tagma_matrix
