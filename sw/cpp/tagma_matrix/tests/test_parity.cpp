// The interop contract against the Rust reference.
//
// Every other test in this module checks the port against itself: the wire
// reader against the wire writer, the products against a reference written with
// the same accessors. That cannot catch a format or a rounding this port and
// the reference would both have to share, and the wire form exists for a matrix
// another implementation wrote.
//
// The values below came from `sw/rust/matrix` and were compared with this port
// byte for byte when it was written. They are pinned so the contract outlives
// that comparison.
//
// Mirrors the fixtures of sw/rust/matrix/tests/.

#include "common.h"
#include "harness.h"

#include <array>
#include <cstddef>
#include <cstdint>
#include <optional>

namespace {

constexpr std::size_t R = 4;
constexpr std::size_t C = 3;
constexpr std::size_t K = 5;
constexpr std::size_t WIRE = 9 + R * C * 5;

using Target = tagma_matrix::Matrix<R, C, tagma_matrix::RowMajor>;

// The stream the reference writes for the shared fixture: value_at(i, j) at row
// i, column j in logical order, with both coordinates and the value carried
// little-endian.
constexpr std::array<uint8_t, WIRE> kReferenceStream{
    0x54, 0x47, 0x4d, 0x58, 0x01, 0x04, 0x00, 0x03, 0x00,  //
    0x00, 0x00, 0x00, 0x00, 0xe2,                          //
    0x00, 0x00, 0x01, 0x00, 0xef,                          //
    0x00, 0x00, 0x02, 0x00, 0xfc,                          //
    0x01, 0x00, 0x00, 0x00, 0xe9,                          //
    0x01, 0x00, 0x01, 0x00, 0xf6,                          //
    0x01, 0x00, 0x02, 0x00, 0x03,                          //
    0x02, 0x00, 0x00, 0x00, 0xf0,                          //
    0x02, 0x00, 0x01, 0x00, 0xfd,                          //
    0x02, 0x00, 0x02, 0x00, 0x0a,                          //
    0x03, 0x00, 0x00, 0x00, 0xf7,                          //
    0x03, 0x00, 0x01, 0x00, 0x04,                          //
    0x03, 0x00, 0x02, 0x00, 0x11,                          //
};

void the_reference_stream_decodes_to_the_shared_fixture() {
  std::optional<tagma_matrix::DecodeError> error;
  const auto matrix = Target::decode(kReferenceStream.data(),
                                     kReferenceStream.size(), error);

  harness::check(matrix.has_value() && !error.has_value(),
                 "the reference stream reads back");
  if (!matrix.has_value()) return;

  for (std::size_t i = 0; i < R; ++i) {
    for (std::size_t j = 0; j < C; ++j) {
      harness::check_eq(matrix->get(i, j), common::value_at(i, j),
                        "the reference stream carries the logical values");
    }
  }
}

void this_port_writes_the_stream_the_reference_writes() {
  const auto matrix = common::row_major<R, C>();
  std::array<uint8_t, WIRE> bytes{};

  harness::check(!matrix.encode_into(bytes.data(), bytes.size()).has_value(),
                 "the buffer holds the matrix");

  for (std::size_t k = 0; k < WIRE; ++k) {
    harness::check_eq(bytes[k], kReferenceStream[k],
                      "the stream matches the reference byte for byte");
  }
}

// The rescale the fixtures share, at the inputs the reference was run on: zero,
// both signs, the half cases, and both ends of the signed thirty-two bit range.
constexpr std::array<int32_t, 10> kAccumulators{
    0, 1, -1, 2, 300, -300, 1'000'000, -1'000'000, INT32_MIN, INT32_MAX};
constexpr std::array<int8_t, 10> kRescaled{0,  0,   0,    1,
                                           87, -87, 127, -128,
                                           -128, 127};

void the_rescale_matches_the_reference_at_the_same_inputs() {
  const auto rescale = common::requant();

  harness::check_eq(rescale.multiplier(), 38'997'123,
                    "the fixture's multiplier");
  harness::check_eq(rescale.shift(), 27u, "the fixture's shift");

  for (std::size_t i = 0; i < kAccumulators.size(); ++i) {
    harness::check_eq(rescale.requantize(kAccumulators[i]), kRescaled[i],
                      "the rescale matches the reference");
  }
}

// The widest intermediate, which is where a width or rounding difference between
// the two implementations would show first.
constexpr std::array<int32_t, 6> kWidestAccumulators{
    INT32_MIN, INT32_MIN + 1, -1, 0, 1, INT32_MAX};
constexpr std::array<int8_t, 6> kWidestRescaled{1, 1, 0, 0, 0, -1};

void the_widest_shift_matches_the_reference() {
  const tagma_matrix::Requant widest(INT32_MIN,
                                     tagma_matrix::Requant::kMaxShift);

  for (std::size_t i = 0; i < kWidestAccumulators.size(); ++i) {
    harness::check_eq(widest.requantize(kWidestAccumulators[i]),
                      kWidestRescaled[i],
                      "the widest shift matches the reference");
  }
}

constexpr std::array<int32_t, R> kReferenceProduct{-8, 6, 20, 34};

void the_vector_product_matches_the_reference_at_the_same_inputs() {
  const auto a = common::row_major<R, C>();
  const std::array<int8_t, C> x{1, -2, 3};
  std::array<int32_t, R> y{};

  tagma_matrix::gemv(a, x, y);

  for (std::size_t i = 0; i < R; ++i) {
    harness::check_eq(y[i], kReferenceProduct[i],
                      "the vector product matches the reference");
  }
}

constexpr std::array<int32_t, R * C> kReferenceContraction{
    1440, -1790, 870,  1215, -1590, 720,
    990,  -1390, 570,  765,  -1190, 420};

void the_contraction_matches_the_reference_at_the_same_inputs() {
  tagma_matrix::Matrix<R, K, tagma_matrix::RowMajor>::Buffer left{};
  for (std::size_t i = 0; i < R; ++i) {
    for (std::size_t j = 0; j < K; ++j) {
      left[i][j] = static_cast<int8_t>(
          (static_cast<int32_t>(i) * 5 - static_cast<int32_t>(j) * 9) % 47 - 20);
    }
  }
  tagma_matrix::Matrix<K, C, tagma_matrix::RowMajor>::Buffer right{};
  for (std::size_t i = 0; i < K; ++i) {
    for (std::size_t j = 0; j < C; ++j) {
      right[i][j] = static_cast<int8_t>(
          (static_cast<int32_t>(i) * 3 + static_cast<int32_t>(j) * 17) % 31 - 15);
    }
  }

  const tagma_matrix::Matrix<R, K, tagma_matrix::RowMajor> a(left);
  const tagma_matrix::Matrix<K, C, tagma_matrix::RowMajor> b(right);
  std::array<std::array<int32_t, C>, R> out{};

  tagma_matrix::matmul(a, b, out);

  for (std::size_t i = 0; i < R; ++i) {
    for (std::size_t j = 0; j < C; ++j) {
      harness::check_eq(out[i][j], kReferenceContraction[i * C + j],
                        "the contraction matches the reference");
    }
  }
}

}  // namespace

int main() {
  the_reference_stream_decodes_to_the_shared_fixture();
  this_port_writes_the_stream_the_reference_writes();
  the_rescale_matches_the_reference_at_the_same_inputs();
  the_widest_shift_matches_the_reference();
  the_vector_product_matches_the_reference_at_the_same_inputs();
  the_contraction_matches_the_reference_at_the_same_inputs();
  return harness::finish("tagma_matrix parity");
}
