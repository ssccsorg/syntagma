#pragma once

// CoordSet: a bit array over the coordinate space. Insert, remove, and
// membership are single-bit operations; union, intersection, difference,
// and symmetric difference are bitwise over the 11,172 slots. Mirrors the
// Rust CoordSet in sw/rust/core.

#include "tagma_core/coord.h"

#include <bitset>

namespace tagma {

class CoordSet {
public:
  static constexpr int kCapacity = Coord::kNValid;

  CoordSet() = default;

  // Inserts coord. Returns true when coord was not already present.
  bool insert(Coord coord) {
    const std::size_t index = coord.index();
    const bool present = bits_.test(index);
    bits_.set(index);
    return !present;
  }

  // Removes coord. Returns true when coord was present.
  bool remove(Coord coord) {
    const std::size_t index = coord.index();
    const bool present = bits_.test(index);
    bits_.reset(index);
    return present;
  }

  bool contains(Coord coord) const { return bits_.test(coord.index()); }

  void clear() { bits_.reset(); }

  // The number of elements (popcount).
  std::size_t len() const { return bits_.count(); }

  bool is_empty() const { return bits_.none(); }

  // The maximum number of elements, always 11,172.
  static constexpr int capacity() { return kCapacity; }

  CoordSet set_union(const CoordSet& other) const {
    CoordSet result;
    result.bits_ = bits_ | other.bits_;
    return result;
  }

  CoordSet set_intersection(const CoordSet& other) const {
    CoordSet result;
    result.bits_ = bits_ & other.bits_;
    return result;
  }

  CoordSet set_difference(const CoordSet& other) const {
    CoordSet result;
    result.bits_ = bits_ & ~other.bits_;
    return result;
  }

  CoordSet set_symmetric_difference(const CoordSet& other) const {
    CoordSet result;
    result.bits_ = bits_ ^ other.bits_;
    return result;
  }

  // True when every element of this set is in other.
  bool is_subset(const CoordSet& other) const {
    return (bits_ & ~other.bits_).none();
  }

  bool is_superset(const CoordSet& other) const {
    return other.is_subset(*this);
  }

  // True when the sets share no element.
  bool is_disjoint(const CoordSet& other) const {
    return (bits_ & other.bits_).none();
  }

private:
  std::bitset<kCapacity> bits_;
};

}  // namespace tagma
