#pragma once

// Keyed hashing for the tagma-sec layer.
//
// The Rust port uses blake3 keyed hashing. This self-contained C++ port uses
// SHA-256 for payload commitments and RFC 2104 HMAC-SHA-256 for keyed tags:
// the 32-byte key is zero-padded to the 64-byte SHA-256 block, and the block
// is mixed with the inner (0x36) and outer (0x5c) pad constants. The
// interface and the semantics are unchanged from the Rust port: a 32-byte
// keyed commitment over the concatenated parts. The underlying hash is a
// porting difference; the interfaces and the security semantics (tamper
// evidence, keyed binding, recompute-and-compare verification) are identical.

#include <array>
#include <cstddef>
#include <cstdint>
#include <vector>

namespace tagma_sec {

using Bytes = std::vector<uint8_t>;

// Unkeyed SHA-256 over `data`, used for audit payload commitments. Mirrors
// the plain blake3::hash of the Rust audit module.
std::array<uint8_t, 32> sha256(const Bytes& data);

// RFC 2104 HMAC-SHA-256 over the concatenation of `parts` under a 32-byte
// `key`. Mirrors keyed_tag in the Rust hash module.
std::array<uint8_t, 32> keyed_tag(const std::array<uint8_t, 32>& key,
                                  const std::vector<Bytes>& parts);

// Constant-time equality over two 32-byte tags: every byte is inspected and
// combined without an early exit, so a mismatching tag does not reveal where
// the difference lies. All verify paths compare tags through this helper
// rather than std::array::operator==.
bool tag_equal(const std::array<uint8_t, 32>& a, const std::array<uint8_t, 32>& b);

// Little-endian byte encodings used by seals and receipts.
Bytes le16(uint16_t v);
Bytes le64(uint64_t v);

}  // namespace tagma_sec
