#pragma once

// CoordPath: an index path, a compile-time N-element array of coordinates.
// The path is an address, not a hash key. Mirrors the Rust CoordPath<N>
// in sw/rust/core.

#include "tagma_core/coord.h"

#include <array>

namespace tagma {

template <int N>
class CoordPath {
public:
  static constexpr int kLength = N;

  // Default-constructs to a path of valid zero coordinates.
  CoordPath() = default;

  explicit CoordPath(const std::array<Coord, N>& coords) : coords_(coords) {}

  static CoordPath from_array(const std::array<Coord, N>& coords) {
    return CoordPath(coords);
  }

  // The internal coordinate array.
  const std::array<Coord, N>& coords() const { return coords_; }

  // The coordinate at index, or nullptr when out of range.
  const Coord* get(int index) const {
    return index >= 0 && index < N ? &coords_[index] : nullptr;
  }

  // The path length, always N.
  static constexpr int len() { return N; }

  static constexpr bool is_empty() { return N == 0; }

  auto begin() const { return coords_.begin(); }
  auto end() const { return coords_.end(); }

private:
  std::array<Coord, N> coords_;
};

}  // namespace tagma
