// Relocation invariance.
//
// The same logical matrix in two physical orders must answer the same
// coordinates with the same values, produce the same product, and write the
// same bytes. This is what makes the teleport claim falsifiable: if an address
// depended on where the bytes sit, these runs would differ.
//
// Mirrors sw/rust/matrix/tests/relocation.rs.

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

void two_physical_orders_answer_the_same_coordinates() {
  const auto rows = common::row_major<R, C>();
  const auto columns = common::col_major<R, C>();

  for (std::size_t i = 0; i < R; ++i) {
    for (std::size_t j = 0; j < C; ++j) {
      harness::check(rows.path_of(i, j) == columns.path_of(i, j),
                     "two orders name the same coordinate");
      const auto row_path = rows.path_of(i, j);
      const auto column_path = columns.path_of(i, j);
      harness::check(row_path.has_value() &&
                         rows.at(*row_path) ==
                             std::optional<int8_t>(common::value_at(i, j)),
                     "the row-major matrix reads the logical value");
      harness::check(column_path.has_value() &&
                         columns.at(*column_path) ==
                             std::optional<int8_t>(common::value_at(i, j)),
                     "the column-major matrix reads the logical value");
    }
  }
}

void two_physical_orders_produce_a_byte_identical_product() {
  const auto rows = common::row_major<R, C>();
  const auto columns = common::col_major<R, C>();
  const auto x = common::activation<C>();

  std::array<int32_t, R> from_rows{};
  std::array<int32_t, R> from_columns{};
  tagma_matrix::gemv(rows, x, from_rows);
  tagma_matrix::gemv(columns, x, from_columns);

  harness::check(from_rows == from_columns,
                 "two physical orders produce one product");
}

// The rescale is what a model's next layer consumes, so the invariance has to
// hold of the element rather than only of the accumulator it came from.
void two_physical_orders_produce_a_byte_identical_requantized_product() {
  const auto rows = common::row_major<R, C>();
  const auto columns = common::col_major<R, C>();
  const auto x = common::activation<C>();
  const auto rescale = common::requant();

  std::array<int8_t, R> from_rows{};
  std::array<int8_t, R> from_columns{};
  tagma_matrix::gemv_requantized(rows, x, rescale, from_rows);
  tagma_matrix::gemv_requantized(columns, x, rescale, from_columns);

  harness::check(from_rows == from_columns,
                 "two physical orders produce one requantized product");
}

void the_requantized_product_is_the_rescale_of_the_product() {
  const auto rows = common::row_major<R, C>();
  const auto x = common::activation<C>();
  const auto rescale = common::requant();

  std::array<int32_t, R> accumulators{};
  tagma_matrix::gemv(rows, x, accumulators);

  std::array<int8_t, R> expected{};
  for (std::size_t i = 0; i < R; ++i) {
    expected[i] = rescale.requantize(accumulators[i]);
  }

  std::array<int8_t, R> from_rows{};
  tagma_matrix::gemv_requantized(rows, x, rescale, from_rows);

  harness::check(from_rows == expected,
                 "the requantized product is the rescale of the product");
}

void a_round_trip_carries_the_matrix_into_another_order() {
  const auto rows = common::row_major<R, C>();
  const auto x = common::activation<C>();

  std::array<uint8_t, WIRE> bytes{};
  const auto written = rows.encode_into(bytes.data(), bytes.size());
  harness::check(!written.has_value(), "the buffer holds the matrix");
  // The stream's length is `encoded_len()`, a constant of the shape, so there is
  // no runtime count to compare against it.

  std::optional<tagma_matrix::DecodeError> error;
  const auto decoded =
      tagma_matrix::Matrix<R, C, tagma_matrix::ColMajor>::decode(bytes.data(),
                                                                bytes.size(), error);
  harness::check(decoded.has_value() && !error.has_value(),
                 "a stream this format wrote reads back");
  if (!decoded.has_value()) return;
  const auto& columns = *decoded;

  for (std::size_t i = 0; i < R; ++i) {
    for (std::size_t j = 0; j < C; ++j) {
      const auto path = columns.path_of(i, j);
      harness::check(path.has_value() &&
                         columns.at(*path) ==
                             std::optional<int8_t>(common::value_at(i, j)),
                     "the relocated matrix carries the logical values");
    }
  }

  std::array<int32_t, R> from_rows{};
  std::array<int32_t, R> from_columns{};
  tagma_matrix::gemv(rows, x, from_rows);
  tagma_matrix::gemv(columns, x, from_columns);
  harness::check(from_rows == from_columns,
                 "the relocated matrix produces the same product");
}

void the_stream_does_not_depend_on_the_physical_order() {
  const auto rows = common::row_major<R, C>();
  const auto columns = common::col_major<R, C>();

  std::array<uint8_t, WIRE> from_rows{};
  std::array<uint8_t, WIRE> from_columns{};
  harness::check(!rows.encode_into(from_rows.data(), from_rows.size()).has_value(),
                 "the buffer holds the row-major matrix");
  harness::check(
      !columns.encode_into(from_columns.data(), from_columns.size()).has_value(),
      "the buffer holds the column-major matrix");

  harness::check(from_rows == from_columns,
                 "two physical orders write the same bytes");
}

void a_relocated_value_writes_the_same_bytes_it_arrived_in() {
  const auto rows = common::row_major<R, C>();

  std::array<uint8_t, WIRE> sent{};
  harness::check(!rows.encode_into(sent.data(), sent.size()).has_value(),
                 "the buffer holds the matrix");

  std::optional<tagma_matrix::DecodeError> error;
  const auto received =
      tagma_matrix::Matrix<R, C, tagma_matrix::ColMajor>::decode(sent.data(),
                                                                sent.size(), error);
  harness::check(received.has_value() && !error.has_value(),
                 "a stream this format wrote reads back");
  if (!received.has_value()) return;

  std::array<uint8_t, WIRE> resent{};
  harness::check(
      !received->encode_into(resent.data(), resent.size()).has_value(),
      "the buffer holds the received matrix");

  harness::check(sent == resent, "a relocated value writes the same bytes");
}

}  // namespace

int main() {
  two_physical_orders_answer_the_same_coordinates();
  two_physical_orders_produce_a_byte_identical_product();
  two_physical_orders_produce_a_byte_identical_requantized_product();
  the_requantized_product_is_the_rescale_of_the_product();
  a_round_trip_carries_the_matrix_into_another_order();
  the_stream_does_not_depend_on_the_physical_order();
  a_relocated_value_writes_the_same_bytes_it_arrived_in();
  return harness::finish("tagma_matrix relocation");
}
