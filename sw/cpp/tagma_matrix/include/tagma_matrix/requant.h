#pragma once

// tagma_matrix: the rescale between an accumulator and an element. Mirrors
// sw/rust/matrix/src/requant.rs.
//
// A product accumulates in int32 and the next layer consumes int8. The step
// between them is one multiply by a fixed point value followed by a right
// shift, and it is where a quantized model's arithmetic is easiest to get
// subtly wrong: the product does not fit in thirty-two bits, and a rounding
// that is correct for a magnitude is wrong for a signed value.
//
// The contract is stated as one result, so any implementation that reproduces
// it is conformant whatever it does inside. The result is the accumulator times
// the multiplier, plus a rounding constant of half the shifted width, moved
// right by the shift, and saturated into int8. The intermediate is sixty-four
// bits wide.
//
// The shift is written here as a floor division rather than as `>>` on a signed
// value, because a right shift of a negative operand is implementation-defined
// in C++17 where the Rust reference states an arithmetic shift.

#include <cstddef>
#include <cstdint>
#include <stdexcept>

namespace tagma_matrix {
namespace detail {

// Division of a possibly negative value by a power of two, flooring.
constexpr int64_t floor_pow2(int64_t value, uint32_t shift) {
  if (shift == 0) return value;
  const int64_t divisor = static_cast<int64_t>(1) << shift;
  const int64_t quotient = value / divisor;
  const int64_t remainder = value % divisor;
  return (remainder != 0 && value < 0) ? quotient - 1 : quotient;
}

// The element nearest a wide value, saturating at the ends of the element range.
constexpr int8_t saturate(int64_t value) {
  if (value > INT8_MAX) return INT8_MAX;
  if (value < INT8_MIN) return INT8_MIN;
  return static_cast<int8_t>(value);
}

}  // namespace detail

// The fixed point rescale between an accumulator and an element.
//
// `multiplier` and `shift` are the two halves of one fixed point value: the
// rescale multiplies by `multiplier` and then divides by two to the `shift`. A
// value below one is carried as a small multiplier with a large shift, and a
// value above one as a large multiplier with a small shift.
class Requant {
 public:
  // The largest shift whose rounding constant still fits beside the product.
  //
  // The product of two thirty-two bit values occupies at most sixty-two bits,
  // so a rounding constant derived from a shift above sixty-two pushes the sum
  // past the sixty-four bit intermediate.
  static constexpr uint32_t kMaxShift = 62;

  // A rescale that multiplies by `multiplier` and moves right by `shift` bits.
  //
  // Throws std::invalid_argument if `shift` is above `kMaxShift`.
  constexpr Requant(int32_t multiplier, uint32_t shift)
      : multiplier_(multiplier), shift_(shift) {
    if (shift > kMaxShift) {
      throw std::invalid_argument(
          "tagma-matrix: shift exceeds the sixty-four bit intermediate");
    }
  }

  // The value the accumulator is multiplied by.
  constexpr int32_t multiplier() const { return multiplier_; }

  // The bits the product is moved right.
  constexpr uint32_t shift() const { return shift_; }

  // Equality over both halves, as the Rust derives give.
  friend constexpr bool operator==(const Requant& left, const Requant& right) {
    return left.multiplier_ == right.multiplier_ && left.shift_ == right.shift_;
  }
  friend constexpr bool operator!=(const Requant& left, const Requant& right) {
    return !(left == right);
  }

  // The rescale of one accumulator, saturated into an element.
  //
  // The rounding constant is added before the shift, and the shift floors, so
  // the product is divided by two to the shift and rounded half up: a product
  // exactly half way between two integers moves toward positive infinity. That
  // is the convention the TFLite line states, and it is not symmetric about
  // zero, since half of -1 is 0 rather than -1. A shift of zero adds no
  // rounding constant and takes the product as it stands.
  constexpr int8_t requantize(int32_t accumulator) const {
    const int64_t product =
        static_cast<int64_t>(accumulator) * static_cast<int64_t>(multiplier_);
    const int64_t rounding =
        (shift_ == 0) ? 0 : (static_cast<int64_t>(1) << (shift_ - 1));
    return detail::saturate(detail::floor_pow2(product + rounding, shift_));
  }

 private:
  int32_t multiplier_;
  uint32_t shift_;
};

// The rescale of a run of accumulators, saturated into elements.
//
// Throws std::invalid_argument if the destination is shorter than the run. A
// destination that cannot hold the run is refused rather than filled part way,
// so a layer that under-sizes its output fails where it happens rather than
// leaving the tail at a stale value.
inline void requantize_into(const int32_t* accumulators, std::size_t count,
                            const Requant& requant, int8_t* out,
                            std::size_t out_len) {
  if (out_len < count) {
    throw std::invalid_argument(
        "tagma-matrix: the destination is shorter than the accumulators");
  }
  for (std::size_t i = 0; i < count; ++i) {
    out[i] = requant.requantize(accumulators[i]);
  }
}

}  // namespace tagma_matrix
