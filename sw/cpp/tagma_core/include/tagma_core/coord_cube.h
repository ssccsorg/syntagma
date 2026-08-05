#pragma once

// CoordCube: an interpretation layer over CoordPath that views N
// characters as a D-dimensional grid with R characters per dimension.
// N must equal D * R, enforced at compile time with static_assert (the
// Rust reference enforces it at runtime in from_path). CoordCube never
// modifies or replaces the underlying CoordPath; storage always uses
// CoordPath. Mirrors the Rust CoordCube<N, D, R> in sw/rust/core.

#include "tagma_core/coord.h"
#include "tagma_core/coord_path.h"

#include <array>
#include <cassert>
#include <ostream>

namespace tagma {

template <int N, int D, int R>
class CoordCube {
public:
  static_assert(N == D * R, "CoordCube: N must equal D * R");

  explicit CoordCube(const CoordPath<N>& path) : path_(path) {}

  // Creates a cube from a path. The compile-time constraint N == D * R
  // holds for every instantiation.
  static CoordCube from_path(const CoordPath<N>& path) { return CoordCube(path); }

  // The underlying path, mirroring as_path and into_path.
  const CoordPath<N>& as_path() const { return path_; }
  CoordPath<N> into_path() const { return path_; }

  // The number of spatial dimensions.
  static constexpr int ndim() { return D; }

  // The number of characters per dimension.
  static constexpr int resolution() { return R; }

  // The total number of characters.
  static constexpr int total_characters() { return N; }

  // The R-character path for dimension dim. Asserts dim < D.
  CoordPath<R> axis(int dim) const {
    assert(dim >= 0 && dim < D);
    std::array<Coord, R> coords{};
    for (int i = 0; i < R; ++i) {
      coords[i] = path_.coords()[dim * R + i];
    }
    return CoordPath<R>::from_array(coords);
  }

  // The Coord at character within dimension dim. Asserts dim < D and
  // character < R.
  Coord coord_at(int dim, int character) const {
    assert(dim >= 0 && dim < D);
    assert(character >= 0 && character < R);
    return path_.coords()[dim * R + character];
  }

  // The full coordinate array, mirroring the Rust coords accessor.
  const std::array<Coord, N>& coords() const { return path_.coords(); }

  // Equality delegates to the underlying path, mirroring the Rust
  // PartialEq and Eq impls.
  bool operator==(const CoordCube& other) const { return path_ == other.path_; }
  bool operator!=(const CoordCube& other) const { return !(*this == other); }

private:
  CoordPath<N> path_;
};

// Display of the cube, mirroring the Rust Display impl:
// CoordCube<N, D, R>[(c0, c1) | (c2, c3) | ...]
template <int N, int D, int R>
std::ostream& operator<<(std::ostream& os, const CoordCube<N, D, R>& cube) {
  os << "CoordCube<" << N << ", " << D << ", " << R << ">[";
  for (int dim = 0; dim < D; ++dim) {
    if (dim > 0) os << " | ";
    os << "(";
    for (int i = 0; i < R; ++i) {
      if (i > 0) os << ", ";
      os << cube.coord_at(dim, i);
    }
    os << ")";
  }
  return os << "]";
}

}  // namespace tagma
