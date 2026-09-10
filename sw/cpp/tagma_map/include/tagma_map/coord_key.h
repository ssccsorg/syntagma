#pragma once

// CoordKey: a fixed-size byte-array key that maps injectively to a
// CoordPath of the same length N. Mirrors the Rust CoordKey<N> in
// sw/rust/map/src/coord_gen.rs.

#include "tagma_core/coord.h"
#include "tagma_core/coord_path.h"

#include <array>
#include <cstddef>
#include <cstdint>
#include <stdexcept>
#include <string>

namespace tagma_map {

template <int N>
class CoordKey {
public:
  explicit CoordKey(const std::array<uint8_t, N>& bytes) : bytes_(bytes) {}

  // Converts a string of exactly N bytes. Throws std::invalid_argument on
  // a length mismatch, mirroring the always-on panic of the Rust From
  // impl.
  static CoordKey from_string(const std::string& key) {
    if (key.size() != static_cast<std::size_t>(N)) {
      throw std::invalid_argument("CoordKey: length mismatch");
    }
    std::array<uint8_t, N> bytes{};
    for (int i = 0; i < N; ++i) {
      bytes[i] = static_cast<uint8_t>(key[i]);
    }
    return CoordKey(bytes);
  }

  // The underlying byte array.
  const std::array<uint8_t, N>& as_bytes() const { return bytes_; }

  // The key length, always N.
  static constexpr std::size_t len() { return static_cast<std::size_t>(N); }

  static constexpr bool is_empty() { return N == 0; }

  // The injective CoordPath mapping: each byte maps to one Coord, and N
  // bytes produce a unique path since 11172^N >> 2^(8N).
  tagma::CoordPath<N> to_coord_path() const {
    std::array<tagma::Coord, N> coords{};
    for (int i = 0; i < N; ++i) {
      coords[i] = tagma::Coord::from_index(bytes_[i]).value();
    }
    return tagma::CoordPath<N>::from_array(coords);
  }

  // The byte-space domain of one key character (0..=255). Each byte maps
  // to one Coord; a path index at or above this value cannot be
  // represented as a byte key. Mirrors CoordKey::BYTE_DOMAIN in the Rust
  // tagma-map crate.
  static constexpr uint16_t kByteDomain = 256;

  // Creates a key from a CoordPath: each character index byte is the key
  // byte. Throws std::invalid_argument when a character index is at or
  // above kByteDomain, mirroring the always-on panic of the Rust
  // from_coord_path.
  static CoordKey from_coord_path(const tagma::CoordPath<N>& path) {
    std::array<uint8_t, N> bytes{};
    for (int i = 0; i < N; ++i) {
      if (path.coords()[i].index() >= kByteDomain) {
        throw std::invalid_argument(
            "CoordKey::from_coord_path: index above the byte-space domain");
      }
      bytes[i] = static_cast<uint8_t>(path.coords()[i].index());
    }
    return CoordKey(bytes);
  }

  bool operator==(const CoordKey& other) const {
    return bytes_ == other.bytes_;
  }

  bool operator!=(const CoordKey& other) const { return !(*this == other); }

private:
  std::array<uint8_t, N> bytes_;
};

}  // namespace tagma_map
