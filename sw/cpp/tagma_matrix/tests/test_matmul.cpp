// The product of two matrices.
//
// A contraction is the operation the library exists for, so it is checked
// against a reference that reads both operands through their coordinates,
// against the identity, and across the physical orders of both operands and of
// the destination.
//
// Mirrors sw/rust/matrix/tests/matmul.rs.

#include "common.h"
#include "harness.h"

#include <array>
#include <cstddef>
#include <cstdint>

namespace {

void the_product_matches_the_coordinate_reference() {
  constexpr std::size_t R = 4;
  constexpr std::size_t K = 5;
  constexpr std::size_t C = 3;

  const auto a = common::row_major<R, K>();
  const auto b = common::row_major<K, C>();
  std::array<std::array<int32_t, C>, R> out{};

  tagma_matrix::matmul(a, b, out);

  harness::check(out == common::matmul_reference(a, b),
                 "the product matches the coordinate reference");
}

void an_identity_operand_leaves_the_other_unchanged() {
  constexpr std::size_t R = 4;
  constexpr std::size_t C = 3;

  const auto identity = common::identity<R>();
  const auto b = common::row_major<R, C>();
  std::array<std::array<int32_t, C>, R> out{};

  tagma_matrix::matmul(identity, b, out);

  for (std::size_t i = 0; i < R; ++i) {
    for (std::size_t j = 0; j < C; ++j) {
      harness::check_eq(out[i][j], static_cast<int32_t>(b.get(i, j)),
                        "the identity leaves the operand unchanged");
    }
  }
}

void two_physical_orders_produce_a_byte_identical_product() {
  constexpr std::size_t R = 4;
  constexpr std::size_t K = 5;
  constexpr std::size_t C = 3;

  const auto rows = common::row_major<R, K>();
  const auto columns = common::col_major<R, K>();

  const auto left = common::row_major<K, C>();
  const auto right = common::col_major<K, C>();

  std::array<std::array<int32_t, C>, R> from_rows{};
  std::array<std::array<int32_t, C>, R> from_columns{};
  tagma_matrix::matmul(rows, left, from_rows);
  tagma_matrix::matmul(columns, right, from_columns);

  harness::check(from_rows == from_columns,
                 "two physical orders produce one product");
}

// A contraction one term long, which is the smallest shape the sum can take and
// the one where an off-by-one in the loop bound shows up as a wrong value
// rather than as a missing one.
void an_outer_product_contracts_over_one_term() {
  constexpr std::size_t R = 3;
  constexpr std::size_t C = 4;

  const auto a = common::row_major<R, 1>();
  const auto b = common::row_major<1, C>();
  std::array<std::array<int32_t, C>, R> out{};

  tagma_matrix::matmul(a, b, out);

  harness::check(out == common::matmul_reference(a, b),
                 "an outer product contracts over one term");
}

void the_requantized_product_is_the_rescale_of_the_product() {
  constexpr std::size_t R = 4;
  constexpr std::size_t K = 5;
  constexpr std::size_t C = 3;

  const auto a = common::row_major<R, K>();
  const auto b = common::col_major<K, C>();
  const auto rescale = common::requant();

  std::array<std::array<int32_t, C>, R> accumulators{};
  tagma_matrix::matmul(a, b, accumulators);

  std::array<std::array<int8_t, C>, R> expected{};
  for (std::size_t i = 0; i < R; ++i) {
    tagma_matrix::requantize_into(accumulators[i].data(), C, rescale,
                                  expected[i].data(), C);
  }

  tagma_matrix::Matrix<R, C, tagma_matrix::RowMajor> out;
  tagma_matrix::matmul_requantized(a, b, rescale, out);

  for (std::size_t i = 0; i < R; ++i) {
    for (std::size_t j = 0; j < C; ++j) {
      harness::check_eq(out.get(i, j), expected[i][j],
                        "the requantized product is the rescale of the product");
    }
  }
}

void the_requantized_product_does_not_depend_on_the_destination_order() {
  constexpr std::size_t R = 4;
  constexpr std::size_t K = 5;
  constexpr std::size_t C = 3;

  const auto a = common::row_major<R, K>();
  const auto b = common::row_major<K, C>();
  const auto rescale = common::requant();

  tagma_matrix::Matrix<R, C, tagma_matrix::RowMajor> rows;
  tagma_matrix::Matrix<R, C, tagma_matrix::ColMajor> columns;
  tagma_matrix::matmul_requantized(a, b, rescale, rows);
  tagma_matrix::matmul_requantized(a, b, rescale, columns);

  for (std::size_t i = 0; i < R; ++i) {
    for (std::size_t j = 0; j < C; ++j) {
      harness::check_eq(rows.get(i, j), columns.get(i, j),
                        "the destination order does not change the result");
    }
  }
}

}  // namespace

int main() {
  the_product_matches_the_coordinate_reference();
  an_identity_operand_leaves_the_other_unchanged();
  two_physical_orders_produce_a_byte_identical_product();
  an_outer_product_contracts_over_one_term();
  the_requantized_product_is_the_rescale_of_the_product();
  the_requantized_product_does_not_depend_on_the_destination_order();
  return harness::finish("tagma_matrix matmul");
}
