#pragma once

// Keyed hashing for the tagma-sec layer.
//
// The Rust port uses blake3 keyed hashing. This self-contained C++ port uses
// SHA-256 for payload commitments and HMAC-SHA-256 for keyed tags, with the
// same interface and semantics: a 32-byte keyed commitment over the
// concatenated parts. The underlying hash is a porting difference; the
// interfaces and the security semantics (tamper evidence, keyed binding,
// recompute-and-compare verification) are identical.

#include <array>
#include <cstddef>
#include <cstdint>
#include <vector>

namespace tagma_sec {

using Bytes = std::vector<uint8_t>;

// Unkeyed SHA-256 over `data`, used for audit payload commitments. Mirrors
// the plain blake3::hash of the Rust audit module.
std::array<uint8_t, 32> sha256(const Bytes& data);

// HMAC-SHA-256 over the concatenation of `parts` under `key`. Mirrors
// keyed_tag in the Rust hash module.
std::array<uint8_t, 32> keyed_tag(const std::array<uint8_t, 32>& key,
                                  const std::vector<Bytes>& parts);

// Little-endian byte encodings used by seals and receipts.
Bytes le16(uint16_t v);
Bytes le64(uint64_t v);

}  // namespace tagma_sec
