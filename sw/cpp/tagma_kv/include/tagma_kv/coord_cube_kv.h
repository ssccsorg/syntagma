#pragma once

// CoordCubeKV: spatial queries over the tagma_kv stores. Mirrors the Rust
// CoordCubeKV trait in sw/rust/kv/src/coord_cube_kv.rs: keys are
// interpreted as multi-dimensional coordinates through CoordCube, the
// query region is generated with tagma_geo, and matching entries are
// looked up through kv.get_by_coord_path.

#include "tagma_geo/spatial.h"

#include "tagma_core/coord_cube.h"
#include "tagma_core/coord_path.h"

#include <array>
#include <cstddef>
#include <cstdint>
#include <utility>
#include <vector>

namespace tagma_kv {

// All entries within L-infinity (Chebyshev) distance radius of center,
// interpreted as a CoordCube<N, D, R>. The constraint D * R == N is
// enforced by CoordCube at compile time.
template <int N, int D, int R, typename KV>
std::vector<std::pair<tagma::CoordPath<N>, std::vector<uint8_t>>> proximity(
    const KV& kv, const tagma::CoordPath<N>& center, std::size_t radius) {
  const tagma::CoordCube<N, D, R> cube =
      tagma::CoordCube<N, D, R>::from_path(center);
  // Pre-size with the exact path count to avoid reallocation, mirroring
  // the Rust with_capacity.
  const tagma_geo::BoundingBoxIter<N> box = tagma_geo::proximity(cube, radius);
  std::vector<std::pair<tagma::CoordPath<N>, std::vector<uint8_t>>> results;
  results.reserve(box.count_paths());
  for (const tagma::CoordPath<N>& path : box) {
    auto value = kv.get_by_coord_path(path);
    if (value) results.emplace_back(path, std::move(*value));
  }
  return results;
}

// All entries within a bounding box defined by per-character (min, max)
// ranges. The array size is deduced as std::size_t and converted to the
// int character count used by CoordPath and BoundingBoxIter, matching
// the non-deduced-context handling in tagma_geo::bounding_box.
template <std::size_t SIZE, typename KV>
std::vector<std::pair<tagma::CoordPath<static_cast<int>(SIZE)>,
                      std::vector<uint8_t>>>
bounding_box_range(
    const KV& kv,
    const std::array<std::pair<uint16_t, uint16_t>, SIZE>& ranges) {
  static constexpr int kNumChars = static_cast<int>(SIZE);
  // Pre-size with the exact path count to avoid reallocation, mirroring
  // the Rust with_capacity.
  const tagma_geo::BoundingBoxIter<kNumChars> box(ranges);
  std::vector<std::pair<tagma::CoordPath<kNumChars>, std::vector<uint8_t>>>
      results;
  results.reserve(box.count_paths());
  for (const tagma::CoordPath<kNumChars>& path : box) {
    auto value = kv.get_by_coord_path(path);
    if (value) results.emplace_back(path, std::move(*value));
  }
  return results;
}

}  // namespace tagma_kv
