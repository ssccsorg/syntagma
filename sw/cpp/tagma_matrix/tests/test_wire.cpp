// The reader's refusals.
//
// A stream this format cannot account for is refused with the reason, rather
// than read as something plausible.
//
// Mirrors sw/rust/matrix/tests/wire.rs.

#include "common.h"
#include "harness.h"

#include <array>
#include <cstddef>
#include <cstdint>
#include <optional>
#include <utility>

namespace {

constexpr std::size_t R = 4;
constexpr std::size_t C = 3;
constexpr std::size_t WIRE = 9 + R * C * 5;

using Target = tagma_matrix::Matrix<R, C, tagma_matrix::RowMajor>;

// A stream this format wrote, from the shared fixture.
std::array<uint8_t, WIRE> stream() {
  const auto matrix = common::row_major<R, C>();
  std::array<uint8_t, WIRE> bytes{};
  const auto error = matrix.encode_into(bytes.data(), bytes.size());
  harness::check(!error.has_value(), "the buffer holds the matrix");
  return bytes;
}

// The offset of the first byte of the record at position `k`.
constexpr std::size_t record(std::size_t k) { return 9 + k * 5; }

void a_valid_stream_reads_back() {
  const auto bytes = stream();
  std::optional<tagma_matrix::DecodeError> error;
  const auto matrix = Target::decode(bytes.data(), bytes.size(), error);

  harness::check(matrix.has_value() && !error.has_value(),
                 "a stream this format wrote reads back");
  if (!matrix.has_value()) return;

  for (std::size_t i = 0; i < R; ++i) {
    for (std::size_t j = 0; j < C; ++j) {
      harness::check_eq(matrix->get(i, j), common::value_at(i, j),
                        "the read-back matrix carries the logical values");
    }
  }
}

void a_short_header_is_refused() {
  const auto bytes = stream();
  std::optional<tagma_matrix::DecodeError> error;
  const auto matrix = Target::decode(bytes.data(), 8, error);

  harness::check(!matrix.has_value(), "a short header is refused");
  harness::check(error == tagma_matrix::DecodeError::too_short(),
                 "a short header reports TooShort");
}

void a_foreign_magic_is_refused() {
  auto bytes = stream();
  bytes[0] = static_cast<uint8_t>('X');
  std::optional<tagma_matrix::DecodeError> error;
  const auto matrix = Target::decode(bytes.data(), bytes.size(), error);

  harness::check(!matrix.has_value(), "a foreign magic is refused");
  harness::check(error == tagma_matrix::DecodeError::bad_magic(),
                 "a foreign magic reports BadMagic");
}

void an_unknown_version_is_refused() {
  auto bytes = stream();
  bytes[4] = 9;
  std::optional<tagma_matrix::DecodeError> error;
  const auto matrix = Target::decode(bytes.data(), bytes.size(), error);

  harness::check(!matrix.has_value(), "an unknown version is refused");
  harness::check(error == tagma_matrix::DecodeError::unsupported_version(9),
                 "an unknown version reports UnsupportedVersion");
}

void a_shape_that_disagrees_is_refused() {
  auto bytes = stream();
  bytes[5] = static_cast<uint8_t>((R + 1) & 0xFF);
  bytes[6] = static_cast<uint8_t>(((R + 1) >> 8) & 0xFF);
  std::optional<tagma_matrix::DecodeError> error;
  const auto matrix = Target::decode(bytes.data(), bytes.size(), error);

  harness::check(!matrix.has_value(), "a shape that disagrees is refused");
  harness::check(error == tagma_matrix::DecodeError::shape_mismatch(R + 1, C),
                 "a shape that disagrees reports ShapeMismatch");
}

void a_record_section_that_ends_early_is_refused() {
  const auto bytes = stream();
  std::optional<tagma_matrix::DecodeError> error;
  const auto matrix = Target::decode(bytes.data(), WIRE - 1, error);

  harness::check(!matrix.has_value(),
                 "a record section that ends early is refused");
  harness::check(error == tagma_matrix::DecodeError::truncated(),
                 "a short record section reports Truncated");
}

void trailing_bytes_are_refused() {
  auto bytes = std::array<uint8_t, WIRE + 1>{};
  const auto source = stream();
  for (std::size_t i = 0; i < WIRE; ++i) bytes[i] = source[i];

  std::optional<tagma_matrix::DecodeError> error;
  const auto matrix = Target::decode(bytes.data(), bytes.size(), error);

  harness::check(!matrix.has_value(), "trailing bytes are refused");
  harness::check(error == tagma_matrix::DecodeError::trailing_bytes(1),
                 "trailing bytes report TrailingBytes");
}

void a_coordinate_outside_the_matrix_is_refused() {
  auto bytes = stream();
  const std::size_t last = record(R * C - 1);
  bytes[last] = static_cast<uint8_t>((R + 1) & 0xFF);
  bytes[last + 1] = static_cast<uint8_t>(((R + 1) >> 8) & 0xFF);

  std::optional<tagma_matrix::DecodeError> error;
  const auto matrix = Target::decode(bytes.data(), bytes.size(), error);

  harness::check(!matrix.has_value(), "a coordinate outside the matrix is refused");
  harness::check(error == tagma_matrix::DecodeError::coordinate_out_of_range(),
                 "a coordinate outside the matrix reports CoordinateOutOfRange");
}

void a_record_in_another_position_is_refused() {
  auto bytes = stream();
  for (std::size_t offset = 0; offset < 5; ++offset) {
    std::swap(bytes[record(0) + offset], bytes[record(1) + offset]);
  }

  std::optional<tagma_matrix::DecodeError> error;
  const auto matrix = Target::decode(bytes.data(), bytes.size(), error);

  harness::check(!matrix.has_value(), "a record in another position is refused");
  harness::check(error == tagma_matrix::DecodeError::coordinate_mismatch(),
                 "a record in another position reports CoordinateMismatch");
}

void a_destination_that_is_too_small_is_refused() {
  const auto matrix = common::row_major<R, C>();
  std::array<uint8_t, WIRE - 1> small{};

  const auto error = matrix.encode_into(small.data(), small.size());

  harness::check(error == tagma_matrix::EncodeError{WIRE, WIRE - 1},
                 "a destination that is too small is refused");
}

}  // namespace

int main() {
  a_valid_stream_reads_back();
  a_short_header_is_refused();
  a_foreign_magic_is_refused();
  an_unknown_version_is_refused();
  a_shape_that_disagrees_is_refused();
  a_record_section_that_ends_early_is_refused();
  trailing_bytes_are_refused();
  a_coordinate_outside_the_matrix_is_refused();
  a_record_in_another_position_is_refused();
  a_destination_that_is_too_small_is_refused();
  return harness::finish("tagma_matrix wire");
}
