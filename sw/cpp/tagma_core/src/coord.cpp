#include "tagma_core/coord.h"

#include <array>
#include <cstdint>
#include <ostream>
#include <string>

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

std::string Coord::to_hangul_string() const {
  const uint32_t cp = code_point();
  std::string out;
  if (cp < 0x80) {
    out += static_cast<char>(cp);
  } else if (cp < 0x800) {
    out += static_cast<char>(0xC0 | (cp >> 6));
    out += static_cast<char>(0x80 | (cp & 0x3F));
  } else {
    out += static_cast<char>(0xE0 | (cp >> 12));
    out += static_cast<char>(0x80 | ((cp >> 6) & 0x3F));
    out += static_cast<char>(0x80 | (cp & 0x3F));
  }
  return out;
}

std::array<uint8_t, 2> Coord::to_le_bytes() const {
  return {static_cast<uint8_t>(index_ & 0xFF),
          static_cast<uint8_t>(index_ >> 8)};
}

std::array<uint8_t, 2> Coord::to_be_bytes() const {
  return {static_cast<uint8_t>(index_ >> 8),
          static_cast<uint8_t>(index_ & 0xFF)};
}

std::optional<Coord> Coord::from_le_bytes(
    const std::array<uint8_t, 2>& bytes) {
  return from_index(static_cast<uint16_t>(bytes[0]) |
                    (static_cast<uint16_t>(bytes[1]) << 8));
}

std::optional<Coord> Coord::from_be_bytes(
    const std::array<uint8_t, 2>& bytes) {
  return from_index((static_cast<uint16_t>(bytes[0]) << 8) |
                    static_cast<uint16_t>(bytes[1]));
}

std::ostream& operator<<(std::ostream& os, const Coord& coord) {
  return os << coord.to_hangul_string();
}

}  // namespace tagma
