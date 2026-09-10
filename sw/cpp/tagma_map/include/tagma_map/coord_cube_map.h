#pragma once

// CoordCubeMap: spatial queries over the tagma_map stores. Mirrors the Rust
// CoordCubeMap trait in sw/rust/map/src/coord_cube_map.rs: keys are
// interpreted as multi-dimensional coordinates through CoordCube, the
// query region is generated with tagma_geo, and matching entries are
// looked up through map.get_by_coord_path.

#include "tagma_geo/spatial.h"

#include "tagma_core/coord_cube.h"
#include "tagma_core/coord_path.h"
#include "tagma_map/coord_key.h"

#include <array>
#include <cstddef>
#include <cstdint>
#include <stdexcept>
#include <utility>
#include <vector>

namespace tagma_map {

// All entries within L-infinity (Chebyshev) distance radius of center,
// interpreted as a CoordCube<N, D, R>. The constraint D * R == N is
// enforced by CoordCube at compile time. Queries operate in the byte-space
// domain of the store keys (0..255 per character); a center character at
// or above the domain throws std::invalid_argument instead of wrapping.
template <int N, int D, int R, typename Map>
std::vector<std::pair<tagma::CoordPath<N>, std::vector<uint8_t>>> proximity(
    const Map& map, const tagma::CoordPath<N>& center, std::size_t radius) {
  for (int i = 0; i < N; ++i) {
    if (center.coords()[i].index() >= CoordKey<N>::kByteDomain) {
      throw std::invalid_argument(
          "tagma_map::proximity: center character above the byte-space "
          "domain");
    }
  }
  const tagma::CoordCube<N, D, R> cube =
      tagma::CoordCube<N, D, R>::from_path(center);
  // Pre-size with the exact path count to avoid reallocation, mirroring
  // the Rust with_capacity.
  const tagma_geo::BoundingBoxIter<N> box =
      tagma_geo::proximity_bounded(cube, radius, CoordKey<N>::kByteDomain);
  std::vector<std::pair<tagma::CoordPath<N>, std::vector<uint8_t>>> results;
  results.reserve(box.count_paths());
  for (const tagma::CoordPath<N>& path : box) {
    auto value = map.get_by_coord_path(path);
    if (value) results.emplace_back(path, std::move(*value));
  }
  return results;
}

// All entries within a bounding box defined by per-character (min, max)
// ranges. The array size is deduced as std::size_t and converted to the
// int character count used by CoordPath and BoundingBoxIter, matching
// the non-deduced-context handling in tagma_geo::bounding_box. Range
// bounds at or above the byte-space domain throw std::invalid_argument.
template <std::size_t SIZE, typename Map>
std::vector<std::pair<tagma::CoordPath<static_cast<int>(SIZE)>,
                      std::vector<uint8_t>>>
bounding_box_range(
    const Map& map,
    const std::array<std::pair<uint16_t, uint16_t>, SIZE>& ranges) {
  static constexpr int kNumChars = static_cast<int>(SIZE);
  for (std::size_t i = 0; i < SIZE; ++i) {
    if (ranges[i].first >= CoordKey<kNumChars>::kByteDomain ||
        ranges[i].second >= CoordKey<kNumChars>::kByteDomain) {
      throw std::invalid_argument(
          "tagma_map::bounding_box_range: range above the byte-space "
          "domain");
    }
  }
  // Pre-size with the exact path count to avoid reallocation, mirroring
  // the Rust with_capacity.
  const tagma_geo::BoundingBoxIter<kNumChars> box(ranges);
  std::vector<std::pair<tagma::CoordPath<kNumChars>, std::vector<uint8_t>>>
      results;
  results.reserve(box.count_paths());
  for (const tagma::CoordPath<kNumChars>& path : box) {
    auto value = map.get_by_coord_path(path);
    if (value) results.emplace_back(path, std::move(*value));
  }
  return results;
}

}  // namespace tagma_map
