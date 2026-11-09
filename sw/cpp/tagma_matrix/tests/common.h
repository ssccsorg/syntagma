#pragma once

// Fixtures shared by the tagma_matrix tests. Mirrors
// sw/rust/matrix/tests/common/mod.rs.

#include "tagma_matrix/elements.h"
#include "tagma_matrix/gemv.h"
#include "tagma_matrix/matmul.h"
#include "tagma_matrix/matrix.h"
#include "tagma_matrix/matrix_ref.h"
#include "tagma_matrix/requant.h"

#include <array>
#include <cstddef>
#include <cstdint>
#include <stdexcept>

namespace common {

// The logical value at row `i`, column `j`.
//
// A small signed spread that produces negative products and non-trivial
// accumulation, so an error in the addressing or in the accumulation changes
// the output instead of cancelling out.
inline int8_t value_at(std::size_t i, std::size_t j) {
  return static_cast<int8_t>(
      (static_cast<int32_t>(i) * 7 + static_cast<int32_t>(j) * 13) % 61 - 30);
}

// The logical values laid out row-major.
template <std::size_t R, std::size_t C>
std::array<std::array<int8_t, C>, R> row_major_buffer() {
  std::array<std::array<int8_t, C>, R> data{};
  for (std::size_t i = 0; i < R; ++i) {
    for (std::size_t j = 0; j < C; ++j) {
      data[i][j] = value_at(i, j);
    }
  }
  return data;
}

// The logical matrix in row-major order.
template <std::size_t R, std::size_t C>
tagma_matrix::Matrix<R, C, tagma_matrix::RowMajor> row_major() {
  return tagma_matrix::Matrix<R, C, tagma_matrix::RowMajor>(
      row_major_buffer<R, C>());
}

// The same logical values laid out column-major.
//
// The buffer is filled by this module's own reading of column-major layout, so
// the test does not borrow the library's notion of it.
template <std::size_t R, std::size_t C>
std::array<std::array<int8_t, C>, R> col_major_buffer() {
  std::array<std::array<int8_t, C>, R> data{};
  for (std::size_t j = 0; j < C; ++j) {
    for (std::size_t i = 0; i < R; ++i) {
      const std::size_t offset = j * R + i;
      data[offset / C][offset % C] = value_at(i, j);
    }
  }
  return data;
}

// The same logical matrix in column-major order.
template <std::size_t R, std::size_t C>
tagma_matrix::Matrix<R, C, tagma_matrix::ColMajor> col_major() {
  return tagma_matrix::Matrix<R, C, tagma_matrix::ColMajor>(
      col_major_buffer<R, C>());
}

// A deterministic activation vector.
template <std::size_t C>
std::array<int8_t, C> activation() {
  std::array<int8_t, C> x{};
  for (std::size_t j = 0; j < C; ++j) {
    x[j] = static_cast<int8_t>((static_cast<int32_t>(j) * 11) % 41 - 20);
  }
  return x;
}

// The product written as the plainest possible loop, reading every element
// through its coordinate rather than through the order-resolved index, so the
// two access paths the library offers are checked against a third.
template <std::size_t R, std::size_t C, class O, class E>
std::array<int32_t, R> reference(const tagma_matrix::Elements<E, R, C, O>& a,
                                 const std::array<int8_t, C>& x) {
  std::array<int32_t, R> y{};
  for (std::size_t i = 0; i < R; ++i) {
    int32_t accumulator = 0;
    for (std::size_t j = 0; j < C; ++j) {
      const auto path = a.path_of(i, j);
      if (!path.has_value()) {
        throw std::logic_error("reference: position is outside the matrix");
      }
      const auto weight = a.at(*path);
      if (!weight.has_value()) {
        throw std::logic_error("reference: coordinate names no element");
      }
      accumulator += static_cast<int32_t>(*weight) * static_cast<int32_t>(x[j]);
    }
    y[i] = accumulator;
  }
  return y;
}

// The product of two matrices, written through the coordinate surface.
//
// Both operands are reached through their coordinates rather than through the
// order-resolved index, so the shipped contraction and this one do not share an
// access path.
template <std::size_t R, std::size_t K, std::size_t C, class OA, class OB,
          class EA, class EB>
std::array<std::array<int32_t, C>, R> matmul_reference(
    const tagma_matrix::Elements<EA, R, K, OA>& a,
    const tagma_matrix::Elements<EB, K, C, OB>& b) {
  std::array<std::array<int32_t, C>, R> out{};
  for (std::size_t i = 0; i < R; ++i) {
    for (std::size_t j = 0; j < C; ++j) {
      int32_t accumulator = 0;
      for (std::size_t k = 0; k < K; ++k) {
        const auto left_path = a.path_of(i, k);
        const auto right_path = b.path_of(k, j);
        if (!left_path.has_value() || !right_path.has_value()) {
          throw std::logic_error("reference: the row is in range");
        }
        const auto left = a.at(*left_path);
        const auto right = b.at(*right_path);
        if (!left.has_value() || !right.has_value()) {
          throw std::logic_error("reference: the coordinate names an element");
        }
        accumulator +=
            static_cast<int32_t>(*left) * static_cast<int32_t>(*right);
      }
      out[i][j] = accumulator;
    }
  }
  return out;
}

// The identity of a size, which leaves the other operand of a product
// unchanged.
template <std::size_t N>
tagma_matrix::Matrix<N, N, tagma_matrix::RowMajor> identity() {
  typename tagma_matrix::Matrix<N, N, tagma_matrix::RowMajor>::Buffer data{};
  for (std::size_t i = 0; i < N; ++i) data[i][i] = 1;
  return tagma_matrix::Matrix<N, N, tagma_matrix::RowMajor>(data);
}

// The rescale the tests use.
//
// A value a little below one, carried the way a quantized model carries it: a
// multiplier that fills a signed thirty-two bit integer and a shift that makes
// it a fraction.
inline tagma_matrix::Requant requant() {
  return tagma_matrix::Requant(38'997'123, 27);
}

// A rescale a little above one, so a run covers both sides of the fixed point.
inline tagma_matrix::Requant requant_above_one() {
  return tagma_matrix::Requant(2'147'483, 20);
}

// Wider than the implementation, so the width of the intermediate is under test
// rather than two copies of one formula. The Rust fixture uses i128 for the
// same reason; the C++ port falls back to 64 bits where no 128-bit type exists.
#if defined(__SIZEOF_INT128__)
using wide_int = __int128;
#else
using wide_int = int64_t;
#endif

// Division that floors, which the language's operator does not do for a
// negative numerator.
inline wide_int floor_divide(wide_int numerator, wide_int denominator) {
  const wide_int quotient = numerator / denominator;
  if (numerator % denominator != 0 && (numerator < 0) != (denominator < 0)) {
    return quotient - 1;
  }
  return quotient;
}

// The rescale of one accumulator, written as a floor division.
//
// The library reaches the result through an arithmetic shift, which floors, and
// this reference reaches it through the language's division, which truncates,
// with the floor applied by hand. The two agree only if the rounding is the one
// the library documents, so this checks the direction of the rounding rather
// than only the width of the intermediate.
inline int8_t requant_reference(int32_t accumulator,
                                const tagma_matrix::Requant& rescale) {
  const wide_int product =
      static_cast<wide_int>(accumulator) * static_cast<wide_int>(rescale.multiplier());
  const wide_int divisor = static_cast<wide_int>(1) << rescale.shift();
  const wide_int rounding = divisor / 2;
  const wide_int shifted = floor_divide(product + rounding, divisor);

  if (shifted > 127) return 127;
  if (shifted < -128) return -128;
  return static_cast<int8_t>(shifted);
}

}  // namespace common
