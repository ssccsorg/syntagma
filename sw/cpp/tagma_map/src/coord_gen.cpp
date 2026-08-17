#include "tagma_map/coord_gen.h"

namespace tagma_map {

std::optional<std::vector<uint32_t>> utf8_decode(const std::string& text) {
  std::vector<uint32_t> out;
  out.reserve(text.size());
  std::size_t i = 0;
  const std::size_t size = text.size();
  while (i < size) {
    const unsigned char b0 = static_cast<unsigned char>(text[i]);
    uint32_t code = 0;
    std::size_t extra = 0;
    if (b0 < 0x80) {
      code = b0;
    } else if ((b0 & 0xE0) == 0xC0) {
      code = b0 & 0x1F;
      extra = 1;
    } else if ((b0 & 0xF0) == 0xE0) {
      code = b0 & 0x0F;
      extra = 2;
    } else if ((b0 & 0xF8) == 0xF0) {
      code = b0 & 0x07;
      extra = 3;
    } else {
      return std::nullopt;  // invalid lead byte
    }
    if (i + extra >= size) return std::nullopt;  // truncated sequence
    for (std::size_t k = 1; k <= extra; ++k) {
      const unsigned char bk = static_cast<unsigned char>(text[i + k]);
      if ((bk & 0xC0) != 0x80) return std::nullopt;  // invalid continuation
      code = (code << 6) | (bk & 0x3F);
    }
    if (extra == 1 && code < 0x80) return std::nullopt;    // overlong
    if (extra == 2 && code < 0x800) return std::nullopt;   // overlong
    if (extra == 3 && code < 0x10000) return std::nullopt;  // overlong
    if (code > 0x10FFFF) return std::nullopt;               // out of range
    if (code >= 0xD800 && code <= 0xDFFF) return std::nullopt;  // surrogate
    out.push_back(code);
    i += extra + 1;
  }
  return out;
}

}  // namespace tagma_map
