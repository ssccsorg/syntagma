// The shipped product against a scalar reference, and the coordinate against the
// order-resolved index.
//
// Mirrors sw/rust/matrix/tests/scalar_ref.rs.

#include "common.h"
#include "harness.h"

#include <cstddef>
#include <cstdint>
#include <optional>
#include <type_traits>

namespace {

// A weight table that lives where the linker puts it needs a literal type: a
// matrix constructible and readable in a constant expression, with no pointer
// the Rust type does not have. The CRTP base keeps both properties; the
// polymorphic base it replaced cost them.
constexpr tagma_matrix::Matrix<2, 3, tagma_matrix::RowMajor>::Buffer kWeights{
    {{1, -2, 3}, {4, -5, 6}}};
constexpr tagma_matrix::Matrix<2, 3, tagma_matrix::RowMajor> kConstMatrix(
    kWeights);

static_assert(kConstMatrix.get(0, 1) == -2,
              "a matrix is readable in a constant expression");
static_assert(kConstMatrix.get(1, 2) == 6,
              "a matrix is readable in a constant expression");
static_assert(std::is_trivially_copyable<tagma_matrix::Matrix<2, 3>>::value,
              "a matrix carries no pointer the Rust type does not have");

// The compile-time assertions above are the gate; this reads the same values at
// run time so the suite reports the property rather than only compiling it.
void a_matrix_is_usable_in_a_constant_expression() {
  harness::check_eq(kConstMatrix.get(0, 1), int8_t{-2},
                    "a matrix reads in a constant expression");
  harness::check_eq(kConstMatrix.get(1, 2), int8_t{6},
                    "a matrix reads in a constant expression");
}

void the_product_matches_the_scalar_reference_bit_for_bit() {
  constexpr std::size_t R = 7;
  constexpr std::size_t C = 5;

  const auto a = common::row_major<R, C>();
  const auto x = common::activation<C>();

  std::array<int32_t, R> y{};
  tagma_matrix::gemv(a, x, y);

  harness::check(y == common::reference(a, x),
                 "the product matches the scalar reference bit for bit");
}

void a_coordinate_reads_the_value_the_order_resolved_index_reads() {
  constexpr std::size_t R = 7;
  constexpr std::size_t C = 5;

  const auto a = common::row_major<R, C>();

  for (std::size_t i = 0; i < R; ++i) {
    for (std::size_t j = 0; j < C; ++j) {
      const auto path = a.path_of(i, j);
      harness::check(path.has_value(), "the position has a coordinate");
      if (!path.has_value()) continue;
      const auto value = a.at(*path);
      harness::check(value.has_value(), "the coordinate reads an element");
      if (!value.has_value()) continue;
      harness::check_eq(*value, common::value_at(i, j),
                        "the coordinate reads the logical value");
      harness::check_eq(*value, a.get(i, j),
                        "the coordinate reads the indexed value");
    }
  }
}

void a_position_outside_the_matrix_has_no_coordinate() {
  constexpr std::size_t R = 7;
  constexpr std::size_t C = 5;

  const auto a = common::row_major<R, C>();

  harness::check(!a.path_of(R, 0).has_value(), "a row past the last has no coordinate");
  harness::check(!a.path_of(0, C).has_value(),
                 "a column past the last has no coordinate");

  const auto row = tagma::Coord::from_index(static_cast<uint16_t>(R));
  const auto column = tagma::Coord::from_index(0);
  harness::check(row.has_value() && column.has_value(),
                 "the constructed coordinates are in range");
  if (!row.has_value() || !column.has_value()) return;

  const tagma::CoordPath<2> path = tagma::CoordPath<2>::from_array({*row, *column});
  harness::check(!a.at(path).has_value(),
                 "a coordinate outside the matrix names no element");
}

void a_matrix_can_be_written_through_its_index_surface() {
  constexpr std::size_t R = 3;
  constexpr std::size_t C = 2;

  tagma_matrix::Matrix<R, C, tagma_matrix::RowMajor> a;
  for (std::size_t i = 0; i < R; ++i) {
    for (std::size_t j = 0; j < C; ++j) {
      a.set(i, j, common::value_at(i, j));
    }
  }

  for (std::size_t i = 0; i < R; ++i) {
    for (std::size_t j = 0; j < C; ++j) {
      harness::check_eq(a.get(i, j), common::value_at(i, j),
                        "the index surface reads what it wrote");
    }
  }
}

// A matrix copies by value, as the family's other fixed-size types do, so a
// copy is a second buffer rather than a second name for one.
void a_copy_is_independent_of_its_source() {
  auto a = common::row_major<2, 3>();
  auto b = a;

  harness::check(a == b, "a copy equals its source");

  a.set(0, 0, 99);
  b.set(1, 2, -99);

  harness::check(a.get(0, 0) == 99, "the source keeps its own write");
  harness::check(b.get(0, 0) == common::value_at(0, 0),
                 "the copy keeps the original value");
  harness::check(b.get(1, 2) == -99, "the copy keeps its own write");
  harness::check(a.get(1, 2) == common::value_at(1, 2),
                 "the source keeps the original value");
}

}  // namespace

int main() {
  a_matrix_is_usable_in_a_constant_expression();
  the_product_matches_the_scalar_reference_bit_for_bit();
  a_coordinate_reads_the_value_the_order_resolved_index_reads();
  a_position_outside_the_matrix_has_no_coordinate();
  a_matrix_can_be_written_through_its_index_surface();
  a_copy_is_independent_of_its_source();
  return harness::finish("tagma_matrix scalar_ref");
}
