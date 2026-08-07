#pragma once

// tagma_kv: hashless string-key store, the C++ port of the Rust tagma-kv
// crate (sw/rust/kv). This header mirrors coord_gen.rs: string to
// CoordPath conversion strategies with zero hash and zero collision.

#include "tagma_core/coord.h"
#include "tagma_core/coord_path.h"

#include <array>
#include <cstddef>
#include <cstdint>
#include <optional>
#include <string>
#include <vector>

namespace tagma_kv {

// Generation failure, mirroring the Rust GenError. Empty keys are rejected
// by every strategy; length mismatches are reported by CoordKey.
enum class GenError {
  kEmptyKey,
  kKeyTooLong,
};

// Decodes a UTF-8 string into Unicode scalar values. Returns nullopt on
// invalid UTF-8. Used by CharWise.
std::optional<std::vector<uint32_t>> utf8_decode(const std::string& text);

// ── Strategies ────────────────────────────────────────────────────────

// Byte-wise dynamic strategy: each UTF-8 byte maps to one Coord. Byte
// values 0..255 are always inside the valid Coord range, so the mapping is
// injective and collision-free. Path length equals key.size().
struct ByteWise {
  static constexpr const char* name() { return "byte-wise"; }

  // Returns nullopt for empty keys.
  static std::optional<std::vector<tagma::Coord>> generate(
      const std::string& key) {
    if (key.empty()) return std::nullopt;
    std::vector<tagma::Coord> coords;
    coords.reserve(key.size());
    for (const unsigned char byte : key) {
      coords.push_back(tagma::Coord::from_index(byte).value());
    }
    return coords;
  }

  static constexpr bool is_injective() { return true; }
  static constexpr std::optional<std::size_t> fixed_depth() {
    return std::nullopt;
  }
};

// Char-wise dynamic strategy: each Unicode scalar value maps to two Coords
// via c0 = cp / 11172, c1 = cp % 11172. Since 11172 * 100 > 0x10FFFF, every
// scalar produces a unique pair, so the mapping is injective. Path length
// equals 2 * (number of scalar values).
struct CharWise {
  static constexpr const char* name() { return "char-wise"; }

  static std::optional<std::vector<tagma::Coord>> generate(
      const std::string& key) {
    if (key.empty()) return std::nullopt;
    const auto code_points = utf8_decode(key);
    if (!code_points) return std::nullopt;
    std::vector<tagma::Coord> coords;
    coords.reserve(code_points->size() * 2);
    for (const uint32_t code : *code_points) {
      const uint16_t c0 = static_cast<uint16_t>(code / tagma::Coord::kNValid);
      const uint16_t c1 = static_cast<uint16_t>(code % tagma::Coord::kNValid);
      coords.push_back(tagma::Coord::from_index(c0).value());
      coords.push_back(tagma::Coord::from_index(c1).value());
    }
    return coords;
  }

  static constexpr bool is_injective() { return true; }
  static constexpr std::optional<std::size_t> fixed_depth() {
    return std::nullopt;
  }
};

// Static prefix strategy: the first N bytes map to N Coords; shorter keys
// are zero-padded. Lossy truncation: keys sharing the first N bytes
// collide. Path length always equals N.
template <int N>
struct Prefix {
  static_assert(N > 0, "Prefix<0> is meaningless; use N >= 1");

  static constexpr const char* name() { return "prefix"; }

  static std::optional<std::vector<tagma::Coord>> generate(
      const std::string& key) {
    if (key.empty()) return std::nullopt;
    std::vector<tagma::Coord> coords;
    coords.reserve(N);
    for (int i = 0; i < N; ++i) {
      const unsigned char byte = i < static_cast<int>(key.size())
                                     ? static_cast<unsigned char>(key[i])
                                     : 0;
      coords.push_back(tagma::Coord::from_index(byte).value());
    }
    return coords;
  }

  static constexpr bool is_injective() { return false; }
  static constexpr std::optional<std::size_t> fixed_depth() {
    return static_cast<std::size_t>(N);
  }
};

// Static byte-fold strategy: XOR-folds all key bytes into N accumulators
// (accumulator j collects bytes at positions i % N == j), then maps each
// accumulator modulo 11172. Lossy compression; path length always N.
template <int N>
struct ByteFold {
  static_assert(N > 0, "ByteFold<0> is meaningless; use N >= 1");

  static constexpr const char* name() { return "byte-fold"; }

  static std::optional<std::vector<tagma::Coord>> generate(
      const std::string& key) {
    if (key.empty()) return std::nullopt;
    std::array<uint16_t, N> acc{};
    for (std::size_t i = 0; i < key.size(); ++i) {
      acc[i % N] ^= static_cast<uint16_t>(key[i]);
    }
    std::vector<tagma::Coord> coords;
    coords.reserve(N);
    for (const uint16_t value : acc) {
      coords.push_back(
          tagma::Coord::from_index(value % tagma::Coord::kNValid).value());
    }
    return coords;
  }

  static constexpr bool is_injective() { return false; }
  static constexpr std::optional<std::size_t> fixed_depth() {
    return static_cast<std::size_t>(N);
  }
};

// The default dynamic strategy.
using DefaultDynamic = ByteWise;

// Converts a string key to Coords via ByteWise. Returns nullopt for empty
// strings.
inline std::optional<std::vector<tagma::Coord>> string_to_coord_path(
    const std::string& key) {
  return ByteWise::generate(key);
}

}  // namespace tagma_kv
