// The rescale's contract.
//
// Three things have to hold for a quantized model's arithmetic to be
// reproducible: the intermediate is wider than thirty-two bits, the rounding
// moves half toward positive infinity rather than away from zero, and a result
// beyond the element range lands on its end rather than wrapping. Each has a
// test here, and the whole surface is also checked against a reference written
// as a floor division, so the direction of the rounding is under test rather
// than only the width.
//
// Mirrors sw/rust/matrix/tests/requant.rs.

#include "common.h"
#include "harness.h"

#include <array>
#include <cstddef>
#include <cstdint>

namespace {

// Accumulators that cover zero, both signs, the half cases, both ends of the
// signed thirty-two bit range, and the largest magnitude a contraction over the
// coordinate space can reach.
constexpr std::array<int32_t, 16> kAccumulators{
    INT32_MIN,    INT32_MIN + 1, -181'612'032, -1'000'000, -300, -3,
    -2,           -1,            0,            1,          2,    3,
    300,          1'000'000,     181'612'032,  INT32_MAX};

// Multipliers that cover zero, the identity, both signs, and both ends of the
// range.
constexpr std::array<int32_t, 8> kMultipliers{0,
                                              1,
                                              -1,
                                              14'508'993,
                                              38'997'123,
                                              2'147'483,
                                              INT32_MIN,
                                              INT32_MAX};

// Shifts that cover no shift, both rounding halves that a narrow width cannot
// see, the ordinary model range, and the bound.
constexpr std::array<uint32_t, 12> kShifts{
    0, 1, 2, 7, 15, 16, 27, 30, 31, 32, 47, tagma_matrix::Requant::kMaxShift};

void the_rescale_agrees_with_a_floor_division_reference() {
  for (const int32_t multiplier : kMultipliers) {
    for (const uint32_t shift : kShifts) {
      const tagma_matrix::Requant rescale(multiplier, shift);
      for (const int32_t accumulator : kAccumulators) {
        harness::check_eq(rescale.requantize(accumulator),
                          common::requant_reference(accumulator, rescale),
                          "the rescale agrees with the floor-division reference");
      }
    }
  }
}

// The product of a thousand by the multiplier of a real model is fourteen
// billion, which a thirty-two bit intermediate cannot hold. A build that
// truncated it would still pass a comparison of small values, so this fixes one
// case where the low bits decide the answer.
void a_product_beyond_thirty_two_bits_keeps_its_low_bits() {
  const tagma_matrix::Requant rescale(14'508'993, 30);
  harness::check(rescale.requantize(1'000) == 14, "the low bits of a wide product");
  harness::check(rescale.requantize(-1'000) == -14,
                 "the low bits of a wide negative product");
}

// Half of an odd negative is exactly between two integers, and where it lands is
// the whole difference between rounding half up and rounding away from zero.
void half_moves_toward_positive_infinity() {
  const tagma_matrix::Requant rescale(1, 1);

  harness::check(rescale.requantize(1) == 1, "half of one moves up");
  harness::check(rescale.requantize(3) == 2, "three halves move up");
  harness::check(rescale.requantize(-1) == 0, "half of minus one moves up to zero");
  harness::check(rescale.requantize(-3) == -1, "minus three halves move up");
}

void a_shift_of_zero_takes_the_product_as_it_stands() {
  const tagma_matrix::Requant rescale(3, 0);
  harness::check(rescale.requantize(10) == 30, "a zero shift multiplies");
  harness::check(rescale.requantize(-10) == -30, "a zero shift multiplies a negative");
}

void a_product_beyond_the_element_range_lands_on_its_end() {
  const tagma_matrix::Requant wide(INT32_MAX, 0);

  harness::check(wide.requantize(INT32_MAX) == INT8_MAX,
                 "a wide product saturates high");
  harness::check(wide.requantize(INT32_MIN) == INT8_MIN,
                 "a wide product saturates low");

  const tagma_matrix::Requant narrow(1'000'000, 0);
  harness::check(narrow.requantize(1'000'000) == INT8_MAX,
                 "a narrow product saturates high");
  harness::check(narrow.requantize(-1'000'000) == INT8_MIN,
                 "a narrow product saturates low");
}

// The product of the two extremes fills sixty-two bits and the rounding
// constant at the largest shift fills sixty-one, so this is the widest the
// intermediate ever gets. The result is one rather than two, which is what says
// the bound is the shift that is allowed rather than one past it.
void the_largest_shift_keeps_the_product_inside_the_intermediate() {
  const tagma_matrix::Requant rescale(INT32_MIN, tagma_matrix::Requant::kMaxShift);
  harness::check(rescale.requantize(INT32_MIN) == 1,
                 "the largest shift keeps the product inside the intermediate");
}

void a_shift_beyond_the_intermediate_is_refused() {
  harness::check_throws(
      [] {
        const tagma_matrix::Requant rescale(1,
                                            tagma_matrix::Requant::kMaxShift + 1);
        static_cast<void>(rescale);
      },
      "shift exceeds the sixty-four bit intermediate",
      "a shift beyond the intermediate is refused");
}

void a_run_matches_the_rescale_of_each_accumulator() {
  const auto rescale = common::requant();
  std::array<int32_t, 64> accumulators{};
  for (std::size_t i = 0; i < accumulators.size(); ++i) {
    accumulators[i] = static_cast<int32_t>((static_cast<int64_t>(i) - 32) * 5'791);
  }
  std::array<int8_t, 64> elements{};

  tagma_matrix::requantize_into(accumulators.data(), accumulators.size(), rescale,
                                elements.data(), elements.size());

  for (std::size_t i = 0; i < accumulators.size(); ++i) {
    harness::check_eq(elements[i], rescale.requantize(accumulators[i]),
                      "a run matches the rescale of each accumulator");
  }
}

void a_destination_shorter_than_the_accumulators_is_refused() {
  const std::array<int32_t, 4> accumulators{1, 1, 1, 1};
  std::array<int8_t, 3> elements{};

  harness::check_throws(
      [&] {
        tagma_matrix::requantize_into(accumulators.data(), accumulators.size(),
                                      common::requant(), elements.data(),
                                      elements.size());
      },
      "the destination is shorter than the accumulators",
      "a short destination is refused");
}

void a_rescale_above_one_scales_up() {
  const auto rescale = common::requant_above_one();
  harness::check(rescale.requantize(50) == 102, "a rescale above one scales up");
  harness::check(rescale.requantize(-50) == -102,
                 "a rescale above one scales a negative up");
}

}  // namespace

int main() {
  the_rescale_agrees_with_a_floor_division_reference();
  a_product_beyond_thirty_two_bits_keeps_its_low_bits();
  half_moves_toward_positive_infinity();
  a_shift_of_zero_takes_the_product_as_it_stands();
  a_product_beyond_the_element_range_lands_on_its_end();
  the_largest_shift_keeps_the_product_inside_the_intermediate();
  a_shift_beyond_the_intermediate_is_refused();
  a_run_matches_the_rescale_of_each_accumulator();
  a_destination_shorter_than_the_accumulators_is_refused();
  a_rescale_above_one_scales_up();
  return harness::finish("tagma_matrix requant");
}
