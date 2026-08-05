#pragma once

// tagma_geo: spatial operations over CoordCube, the C++ port of the Rust
// tagma-geo crate (sw/rust/geo). Mirrors the reference semantics:
// BoundingBoxIter, HammingFilter, and the SpatialOps and DistanceMetrics
// methods as free functions over CoordCube<N, D, R>.
//
// Dimension values are little-endian base-11172 integers; for R >= 5 the
// u64 value wraps, matching the documented Rust limitation.

#include "tagma_core/coord.h"
#include "tagma_core/coord_cube.h"
#include "tagma_core/coord_path.h"

#include <array>
#include <cassert>
#include <cstddef>
#include <cstdint>
#include <limits>
#include <utility>

namespace tagma_geo {

// Non-deduced context helper: prevents template argument deduction from a
// parameter type, so N in bounding_box is deduced only from the cube.
template <typename T>
struct type_identity {
  using type = T;
};

template <typename T>
using type_identity_t = typename type_identity<T>::type;

// ── BoundingBoxIter ───────────────────────────────────────────────────

// Iterates over all CoordPath<N> within per-character (min, max) ranges,
// in mixed-radix order. Mirrors the Rust BoundingBoxIter<N>.
template <int N>
class BoundingBoxIter {
public:
  using value_type = tagma::CoordPath<N>;
  using difference_type = std::ptrdiff_t;
  using pointer = const value_type*;
  using reference = value_type;
  using iterator_category = std::forward_iterator_tag;

  // Creates an iterator over the given per-character ranges. Asserts that
  // every range is non-inverted and max < 11172.
  explicit BoundingBoxIter(
      const std::array<std::pair<uint16_t, uint16_t>, N>& ranges) {
    for (int i = 0; i < N; ++i) {
      assert(ranges[i].first <= ranges[i].second &&
             "BoundingBoxIter: inverted range");
      assert(ranges[i].second < tagma::Coord::kNValid &&
             "BoundingBoxIter: max out of bounds");
    }
    ranges_ = ranges;
    for (int i = 0; i < N; ++i) {
      current_[i] = ranges_[i].first;
    }
    finished_ = (N == 0);
  }

  // True when no more paths remain.
  bool is_empty() const { return finished_; }

  // The total number of paths, the saturating product of range widths.
  std::size_t count_paths() const {
    std::size_t total = 1;
    for (const auto& range : ranges_) {
      const std::size_t width =
          static_cast<std::size_t>(range.second - range.first + 1);
      total = saturating_mul(total, width);
    }
    return total;
  }

  value_type operator*() const {
    std::array<tagma::Coord, N> coords{};
    for (int i = 0; i < N; ++i) {
      coords[i] = tagma::Coord::from_index(current_[i]).value();
    }
    return tagma::CoordPath<N>::from_array(coords);
  }

  BoundingBoxIter& operator++() {
    if (finished_) return *this;
    int pos = N;
    while (pos > 0) {
      --pos;
      if (current_[pos] < ranges_[pos].second) {
        ++current_[pos];
        for (int reset = pos + 1; reset < N; ++reset) {
          current_[reset] = ranges_[reset].first;
        }
        return *this;
      }
      current_[pos] = ranges_[pos].first;
    }
    finished_ = true;
    return *this;
  }

  bool operator==(const BoundingBoxIter& other) const {
    if (finished_ || other.finished_) {
      return finished_ == other.finished_;
    }
    return current_ == other.current_;
  }

  bool operator!=(const BoundingBoxIter& other) const {
    return !(*this == other);
  }

  // Range-for support.
  BoundingBoxIter begin() const { return *this; }
  BoundingBoxIter end() const {
    BoundingBoxIter result = *this;
    result.finished_ = true;
    return result;
  }

private:
  static std::size_t saturating_mul(std::size_t a, std::size_t b) {
    return b != 0 && a > (std::numeric_limits<std::size_t>::max() / b)
               ? std::numeric_limits<std::size_t>::max()
               : a * b;
  }

  std::array<std::pair<uint16_t, uint16_t>, N> ranges_{};
  std::array<uint16_t, N> current_{};
  bool finished_ = false;
};

// ── HammingFilter ─────────────────────────────────────────────────────

// Iterates over paths within a Hamming radius of a center path, filtering
// an underlying BoundingBoxIter. Mirrors the Rust HammingFilter<N>.
template <int N>
class HammingFilter {
public:
  using value_type = tagma::CoordPath<N>;
  using difference_type = std::ptrdiff_t;
  using pointer = const value_type*;
  using reference = value_type;
  using iterator_category = std::forward_iterator_tag;

  HammingFilter(BoundingBoxIter<N> inner, const tagma::CoordPath<N>& center,
                std::size_t max_distance)
      : inner_(inner), center_(center), max_distance_(max_distance) {
    skip_invalid();
  }

  value_type operator*() const { return *inner_; }

  HammingFilter& operator++() {
    ++inner_;
    skip_invalid();
    return *this;
  }

  bool operator==(const HammingFilter& other) const {
    return inner_ == other.inner_;
  }

  bool operator!=(const HammingFilter& other) const {
    return !(*this == other);
  }

  HammingFilter begin() const { return *this; }
  HammingFilter end() const {
    HammingFilter result = *this;
    result.inner_ = result.inner_.end();
    return result;
  }

private:
  std::size_t hamming_to_center(const tagma::CoordPath<N>& candidate) const {
    std::size_t distance = 0;
    for (int i = 0; i < N; ++i) {
      if (candidate.coords()[i] != center_.coords()[i]) ++distance;
    }
    return distance;
  }

  void skip_invalid() {
    while (inner_ != inner_.end() &&
           hamming_to_center(*inner_) > max_distance_) {
      ++inner_;
    }
  }

  BoundingBoxIter<N> inner_;
  tagma::CoordPath<N> center_;
  std::size_t max_distance_;
};

// ── SpatialOps: region generation over CoordCube ──────────────────────

// All CoordPath<N> within per-character (min, max) ranges. The ranges
// parameter is in a non-deduced context because std::array sizes are
// std::size_t while CoordCube template parameters are int.
template <int N, int D, int R>
BoundingBoxIter<N> bounding_box(
    const tagma::CoordCube<N, D, R>&,
    const type_identity_t<std::array<std::pair<uint16_t, uint16_t>, N>>&
        ranges) {
  return BoundingBoxIter<N>(ranges);
}

// All CoordPath<N> within an L-infinity (Chebyshev) proximity radius of
// the cube center, clamped to the valid index range.
template <int N, int D, int R>
BoundingBoxIter<N> proximity(const tagma::CoordCube<N, D, R>& cube,
                             std::size_t radius) {
  std::array<std::pair<uint16_t, uint16_t>, N> ranges{};
  for (int i = 0; i < N; ++i) {
    const std::size_t index = cube.coords()[i].index();
    const std::size_t min = index >= radius ? index - radius : 0;
    const std::size_t max = index + radius > 11171 ? 11171 : index + radius;
    ranges[i] = {static_cast<uint16_t>(min), static_cast<uint16_t>(max)};
  }
  return BoundingBoxIter<N>(ranges);
}

// All CoordPath<N> within a Hamming distance radius of the cube center.
template <int N, int D, int R>
HammingFilter<N> proximity_hamming(const tagma::CoordCube<N, D, R>& cube,
                                   std::size_t radius) {
  const BoundingBoxIter<N> box = proximity(cube, radius > 0 ? radius : 1);
  return HammingFilter<N>(box, cube.as_path(), radius);
}

// ── Internal helpers ──────────────────────────────────────────────────

// The R characters of dimension dim as a little-endian base-11172 integer
// in [0, 11172^R). Wraps (mod 2^64) for R >= 5, as documented.
template <int N, int D, int R>
std::uint64_t dimension_value(const tagma::CoordCube<N, D, R>& cube, int dim) {
  const int start = dim * R;
  std::uint64_t value = 0;
  std::uint64_t mul = 1;
  for (int i = 0; i < R; ++i) {
    const std::uint64_t index = cube.coords()[start + i].index();
    value += index * mul;
    mul *= static_cast<std::uint64_t>(tagma::Coord::kNValid);
  }
  return value;
}

// The maximum possible value for a single dimension, 11172^R - 1. Returns
// 0 for R >= 5 due to u64 overflow, as documented.
template <int R>
std::uint64_t dimension_max_value() {
  std::uint64_t max = 0;
  std::uint64_t mul = 1;
  for (int i = 0; i < R; ++i) {
    max += 11171ull * mul;
    mul *= static_cast<std::uint64_t>(tagma::Coord::kNValid);
  }
  return max;
}

// Newton-Raphson square root approximation.
double sqrt_approx(double x) {
  if (x <= 0.0) return 0.0;
  double guess = x;
  for (int i = 0; i < 12; ++i) {
    guess = (guess + x / guess) * 0.5;
  }
  return guess;
}

// ── DistanceMetrics: measurement between two CoordCubes ───────────────

// Hamming distance: the count of character positions that differ.
template <int N, int D, int R>
std::size_t hamming_distance(const tagma::CoordCube<N, D, R>& a,
                             const tagma::CoordCube<N, D, R>& b) {
  std::size_t distance = 0;
  for (int i = 0; i < N; ++i) {
    if (a.coords()[i] != b.coords()[i]) ++distance;
  }
  return distance;
}

// Axis-wise Hamming distance: per-dimension character differences.
template <int N, int D, int R>
std::array<std::size_t, D> hamming_distance_axes(
    const tagma::CoordCube<N, D, R>& a, const tagma::CoordCube<N, D, R>& b) {
  std::array<std::size_t, D> out{};
  for (int dim = 0; dim < D; ++dim) {
    const int start = dim * R;
    std::size_t diff = 0;
    for (int i = 0; i < R; ++i) {
      if (a.coords()[start + i] != b.coords()[start + i]) ++diff;
    }
    out[dim] = diff;
  }
  return out;
}

// Normalised Euclidean distance approximation in [0, sqrt(D)]. Each
// dimension value is normalised to [0, 1]. Uses a Newton-Raphson square
// root approximation.
template <int N, int D, int R>
double euclidean_distance_approx(const tagma::CoordCube<N, D, R>& a,
                                 const tagma::CoordCube<N, D, R>& b) {
  double sum_sq = 0.0;
  const double max_value = static_cast<double>(dimension_max_value<R>());
  for (int dim = 0; dim < D; ++dim) {
    const double v1 = static_cast<double>(dimension_value<N, D, R>(a, dim));
    const double v2 = static_cast<double>(dimension_value<N, D, R>(b, dim));
    const double diff = (v1 - v2) / max_value;
    sum_sq += diff * diff;
  }
  return sqrt_approx(sum_sq);
}

// Manhattan (L1) distance: the sum of per-dimension absolute differences.
template <int N, int D, int R>
std::uint64_t manhattan_distance(const tagma::CoordCube<N, D, R>& a,
                                 const tagma::CoordCube<N, D, R>& b) {
  std::uint64_t sum = 0;
  for (int dim = 0; dim < D; ++dim) {
    const std::uint64_t v1 = dimension_value<N, D, R>(a, dim);
    const std::uint64_t v2 = dimension_value<N, D, R>(b, dim);
    sum += v1 > v2 ? v1 - v2 : v2 - v1;
  }
  return sum;
}

}  // namespace tagma_geo
