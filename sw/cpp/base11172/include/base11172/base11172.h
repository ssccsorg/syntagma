#pragma once

// base11172: Tagma native serialization format.
//
// Encodes arbitrary byte sequences into self-validating compositional
// character strings using the Tagma coordinate space as the alphabet.
// Self-validating: invalid characters are immediately detectable.
// No special characters: URL-safe, no escaping needed. Deterministic:
// encoding is a pure function of the input. Mirrors the Rust base11172
// crate in sw/rust/base11172.
//
// The Rust API returns a UTF-8 String; this C++ port uses std::u32string
// (code points) so encoding and decoding stay unambiguous.

#include "tagma_core/coord.h"

#include <array>
#include <cstddef>
#include <cstdint>
#include <optional>
#include <string>
#include <vector>

namespace base11172 {

// The number of distinct characters used by this encoding (11172).
constexpr uint32_t kNChars = tagma::Coord::kNValid;

// Encodes a u16 into two compositional characters (base-11172). Each
// character carries log2(11172) ~= 13.45 bits, so a pair covers a u16.
std::array<char32_t, 2> encode_u16(uint16_t value);

// Encodes bytes into a compositional string, 2 bytes per character pair.
// An odd trailing byte is encoded as a pair with an implicit zero high
// byte, mirroring the Rust implementation.
std::u32string encode_bytes(const uint8_t* data, std::size_t size);

// Decodes a pair of compositional characters back to a u16. Returns
// nullopt when either character falls outside the valid compositional
// block (U+AC00..U+D7AF).
std::optional<uint16_t> decode_pair(char32_t c0, char32_t c1);

// Decodes a string back to bytes, 2 characters per u16 pair. Returns
// nullopt when the string has an odd character count or any invalid
// character.
std::optional<std::vector<uint8_t>> decode_bytes(const std::u32string& text);

}  // namespace base11172
