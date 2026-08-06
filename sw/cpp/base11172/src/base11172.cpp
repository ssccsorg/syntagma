#include "base11172/base11172.h"

namespace base11172 {

std::array<char32_t, 2> encode_u16(uint16_t value) {
  const uint32_t hi = static_cast<uint32_t>(value) / kNChars;
  const uint32_t lo = static_cast<uint32_t>(value) % kNChars;
  const tagma::Coord c0 =
      tagma::Coord::from_index(static_cast<uint16_t>(hi)).value();
  const tagma::Coord c1 =
      tagma::Coord::from_index(static_cast<uint16_t>(lo)).value();
  return {c0.to_char(), c1.to_char()};
}

std::u32string encode_bytes(const uint8_t* data, std::size_t size) {
  std::u32string out;
  out.reserve((size + 1) / 2 * 2);
  for (std::size_t i = 0; i < size; i += 2) {
    const uint16_t value =
        (i + 1 < size)
            ? static_cast<uint16_t>(data[i]) |
                  (static_cast<uint16_t>(data[i + 1]) << 8)
            : static_cast<uint16_t>(data[i]);
    const auto [c0, c1] = encode_u16(value);
    out.push_back(c0);
    out.push_back(c1);
  }
  return out;
}

std::optional<uint16_t> decode_pair(char32_t c0, char32_t c1) {
  const auto coord0 = tagma::Coord::from_char(c0);
  if (!coord0) return std::nullopt;
  const auto coord1 = tagma::Coord::from_char(c1);
  if (!coord1) return std::nullopt;
  return static_cast<uint16_t>(static_cast<uint32_t>(coord0->index()) *
                                   kNChars +
                               coord1->index());
}

std::optional<std::vector<uint8_t>> decode_bytes(const std::u32string& text) {
  if (text.size() % 2 != 0) return std::nullopt;
  std::vector<uint8_t> out;
  out.reserve(text.size());
  for (std::size_t i = 0; i < text.size(); i += 2) {
    const auto value = decode_pair(text[i], text[i + 1]);
    if (!value) return std::nullopt;
    out.push_back(static_cast<uint8_t>(*value & 0xFF));
    out.push_back(static_cast<uint8_t>(*value >> 8));
  }
  return out;
}

}  // namespace base11172
