#pragma once

// CoordSetN: a sparse set of coordinates of depth N, backed by the
// CoordSpaceN tree. Memory allocates lazily: only paths that are actually
// inserted consume nodes. For N=1 the dense CoordSet is preferred.
// Set operations walk the tree, O(entries) per walk. Mirrors the Rust
// CoordSetN<N> in sw/rust/core.

#include "tagma_core/coord_path.h"
#include "tagma_core/coord_space_n.h"

#include <cstddef>
#include <utility>
#include <variant>
#include <vector>

namespace tagma {

template <int N>
class CoordSetN {
public:
  using Unit = std::monostate;

  CoordSetN() = default;

  // The number of coordinates in the set.
  std::size_t len() const { return space_.len(); }

  bool is_empty() const { return space_.is_empty(); }

  // True when path is in the set.
  bool contains(const CoordPath<N>& path) const {
    return space_.at_path(path) != nullptr;
  }

  // Inserts path. Returns true when path was newly inserted.
  bool insert(const CoordPath<N>& path) {
    return !space_.place_path(path, Unit{}).has_value();
  }

  // Removes path. Returns true when path was present.
  bool remove(const CoordPath<N>& path) {
    return space_.vacate_path(path).has_value();
  }

  // Removes all coordinates from the set.
  void clear() { space_.clear(); }

  // All paths in depth-first coordinate-ascending order.
  std::vector<CoordPath<N>> paths() const { return space_.paths(); }

  // ── Set operations ─────────────────────────────────────────────────

  CoordSetN set_union(const CoordSetN& other) const {
    CoordSetN result;
    for (const CoordPath<N>& path : paths()) result.insert(path);
    for (const CoordPath<N>& path : other.paths()) result.insert(path);
    return result;
  }

  CoordSetN set_intersection(const CoordSetN& other) const {
    const CoordSetN* smaller = this;
    const CoordSetN* larger = &other;
    if (len() > other.len()) std::swap(smaller, larger);
    CoordSetN result;
    for (const CoordPath<N>& path : smaller->paths()) {
      if (larger->contains(path)) result.insert(path);
    }
    return result;
  }

  CoordSetN set_difference(const CoordSetN& other) const {
    CoordSetN result;
    for (const CoordPath<N>& path : paths()) {
      if (!other.contains(path)) result.insert(path);
    }
    return result;
  }

  CoordSetN set_symmetric_difference(const CoordSetN& other) const {
    CoordSetN result;
    for (const CoordPath<N>& path : paths()) {
      if (!other.contains(path)) result.insert(path);
    }
    for (const CoordPath<N>& path : other.paths()) {
      if (!contains(path)) result.insert(path);
    }
    return result;
  }

  // True when every path of this set is in other.
  bool is_subset(const CoordSetN& other) const {
    for (const CoordPath<N>& path : paths()) {
      if (!other.contains(path)) return false;
    }
    return true;
  }

  bool is_superset(const CoordSetN& other) const {
    return other.is_subset(*this);
  }

  // True when the sets share no path.
  bool is_disjoint(const CoordSetN& other) const {
    const CoordSetN* smaller = this;
    const CoordSetN* larger = &other;
    if (len() > other.len()) std::swap(smaller, larger);
    for (const CoordPath<N>& path : smaller->paths()) {
      if (larger->contains(path)) return false;
    }
    return true;
  }

  // Equality: same length and mutual subset, mirroring the Rust Eq.
  bool operator==(const CoordSetN& other) const {
    return len() == other.len() && is_subset(other);
  }

  bool operator!=(const CoordSetN& other) const { return !(*this == other); }

private:
  CoordSpaceN<N, Unit> space_;
};

}  // namespace tagma
