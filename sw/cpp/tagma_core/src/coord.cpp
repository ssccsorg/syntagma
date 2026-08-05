#include "tagma_core/coord.h"

namespace tagma {

std::optional<Coord> Coord::from_axes(int initial, int medial, int final) {
  if (initial < 0 || initial >= kInitialMax || medial < 0 ||
      medial >= kMedialMax || final < 0 || final >= kFinalMax) {
    return std::nullopt;
  }
  return Coord(static_cast<uint16_t>(initial * (kMedialMax * kFinalMax) +
                                     medial * kFinalMax + final));
}

std::optional<Coord> Coord::from_index(uint16_t index) {
  if (index >= kNValid) return std::nullopt;
  return Coord(index);
}

std::optional<Coord> Coord::from_code_point(uint16_t code_point) {
  if (code_point < kBase || code_point > kLast) return std::nullopt;
  const uint16_t offset = static_cast<uint16_t>(code_point - kBase);
  if (offset >= kNValid) return std::nullopt;  // filler positions
  return Coord(offset);
}

std::optional<Coord> Coord::from_char(char32_t ch) {
  if (ch < kBase || ch > kLast) return std::nullopt;
  const uint16_t offset = static_cast<uint16_t>(ch - kBase);
  if (offset >= kNValid) return std::nullopt;
  return Coord(offset);
}

std::tuple<int, int, int> Coord::axes() const {
  const int index = index_;
  return {index / (kMedialMax * kFinalMax), (index / kFinalMax) % kMedialMax,
          index % kFinalMax};
}

std::tuple<int, int, int> Coord::hamming_distance(const Coord& other) const {
  const auto [i1, m1, f1] = axes();
  const auto [i2, m2, f2] = other.axes();
  const auto abs_diff = [](int a, int b) { return a > b ? a - b : b - a; };
  return {abs_diff(i1, i2), abs_diff(m1, m2), abs_diff(f1, f2)};
}

}  // namespace tagma
