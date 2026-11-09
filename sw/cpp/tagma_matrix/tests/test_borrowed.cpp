// The borrowed matrix.
//
// This is the path weights take when they live in flash: the buffer is not
// copied into RAM, so every read has to reach the same bytes through the same
// address arithmetic an owned matrix uses. The tests therefore check the
// borrowed type against the owned one, against the scalar reference, and across
// the two physical orders.
//
// Mirrors sw/rust/matrix/tests/borrowed.rs.

#include "common.h"
#include "harness.h"

#include <array>
#include <cstddef>
#include <cstdint>
#include <optional>

namespace {

constexpr std::size_t R = 9;
constexpr std::size_t C = 6;
constexpr std::size_t WIRE = 9 + R * C * 5;

using Borrowed = tagma_matrix::MatrixRef<R, C, tagma_matrix::RowMajor>;

void a_borrowed_matrix_answers_what_the_owned_one_answers() {
  const auto owned = common::row_major<R, C>();
  const auto buffer = common::row_major_buffer<R, C>();
  const Borrowed borrowed(buffer);

  for (std::size_t i = 0; i < R; ++i) {
    for (std::size_t j = 0; j < C; ++j) {
      const int8_t expected = common::value_at(i, j);
      harness::check_eq(borrowed.get(i, j), expected, "borrowed element");
      harness::check_eq(borrowed.get(i, j), owned.get(i, j),
                        "borrowed and owned agree");
      harness::check(borrowed.path_of(i, j) == owned.path_of(i, j),
                     "borrowed and owned name the same coordinate");
      const auto path = borrowed.path_of(i, j);
      harness::check(path.has_value() &&
                         borrowed.at(*path) == std::optional<int8_t>(expected),
                     "the coordinate reads the value");
    }
  }
}

void a_borrowed_matrix_produces_what_the_owned_one_produces() {
  const auto owned = common::row_major<R, C>();
  const auto buffer = common::row_major_buffer<R, C>();
  const Borrowed borrowed(buffer);
  const auto x = common::activation<C>();

  std::array<int32_t, R> from_owned{};
  std::array<int32_t, R> from_borrowed{};
  tagma_matrix::gemv(owned, x, from_owned);
  tagma_matrix::gemv(borrowed, x, from_borrowed);

  harness::check(from_borrowed == from_owned, "borrowed product equals owned");
  harness::check(from_borrowed == common::reference(borrowed, x),
                 "borrowed product equals the reference");
}

void a_borrowed_matrix_is_relocation_invariant() {
  const auto rows_buffer = common::row_major_buffer<R, C>();
  const auto columns_buffer = common::col_major_buffer<R, C>();
  const tagma_matrix::MatrixRef<R, C, tagma_matrix::RowMajor> rows(rows_buffer);
  const tagma_matrix::MatrixRef<R, C, tagma_matrix::ColMajor> columns(
      columns_buffer);
  const auto x = common::activation<C>();

  std::array<int32_t, R> from_rows{};
  std::array<int32_t, R> from_columns{};
  tagma_matrix::gemv(rows, x, from_rows);
  tagma_matrix::gemv(columns, x, from_columns);

  harness::check(from_rows == from_columns, "two orders produce one product");
  for (std::size_t i = 0; i < R; ++i) {
    for (std::size_t j = 0; j < C; ++j) {
      harness::check(rows.path_of(i, j) == columns.path_of(i, j),
                     "two orders name the same coordinate");
    }
  }
}

void a_position_outside_a_borrowed_matrix_has_no_coordinate() {
  const auto buffer = common::row_major_buffer<R, C>();
  const Borrowed borrowed(buffer);

  harness::check(!borrowed.path_of(R, 0).has_value(),
                 "a row past the last has no coordinate");
  harness::check(!borrowed.path_of(0, C).has_value(),
                 "a column past the last has no coordinate");
}

// The write path takes a reader, not an owner, so weights that live in flash
// are sent on without being copied into RAM first.
void a_borrowed_matrix_writes_what_the_owned_one_writes() {
  const auto owned = common::row_major<R, C>();
  const auto buffer = common::row_major_buffer<R, C>();
  const Borrowed borrowed(buffer);

  harness::check(Borrowed::encoded_len() ==
                     tagma_matrix::Matrix<R, C, tagma_matrix::RowMajor>::encoded_len(),
                 "borrowed and owned streams are the same length");

  std::array<uint8_t, WIRE> from_owned{};
  std::array<uint8_t, WIRE> from_borrowed{};
  harness::check(!owned.encode_into(from_owned.data(), from_owned.size()).has_value(),
                 "the buffer holds the owned matrix");
  harness::check(
      !borrowed.encode_into(from_borrowed.data(), from_borrowed.size()).has_value(),
      "the buffer holds the borrowed matrix");

  harness::check(from_borrowed == from_owned, "borrowed and owned write the same bytes");
}

// The same refusal an owned matrix gives, so a stream written from a borrowed
// matrix is held to the same length.
void a_borrowed_matrix_refuses_a_destination_that_is_too_small() {
  const auto buffer = common::row_major_buffer<R, C>();
  const Borrowed borrowed(buffer);

  std::array<uint8_t, WIRE - 1> short_buffer{};
  const auto error = borrowed.encode_into(short_buffer.data(), short_buffer.size());

  harness::check(error == tagma_matrix::EncodeError{WIRE, WIRE - 1},
                 "a short destination is refused");
}

}  // namespace

int main() {
  a_borrowed_matrix_answers_what_the_owned_one_answers();
  a_borrowed_matrix_produces_what_the_owned_one_produces();
  a_borrowed_matrix_is_relocation_invariant();
  a_position_outside_a_borrowed_matrix_has_no_coordinate();
  a_borrowed_matrix_writes_what_the_owned_one_writes();
  a_borrowed_matrix_refuses_a_destination_that_is_too_small();
  return harness::finish("tagma_matrix borrowed");
}
