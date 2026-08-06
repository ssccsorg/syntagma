#pragma once

// CoordSet: a bit array over the coordinate space. Insert, remove, and
// membership are single-bit operations; union, intersection, difference,
// and symmetric difference are bitwise over the 11,172 slots. Mirrors the
// Rust CoordSet in sw/rust/core.

#include "tagma_core/coord.h"

#include <bitset>
#include <cassert>
#include <cstddef>
#include <optional>
#include <utility>

namespace tagma {

class CoordSet {
public:
  static constexpr int kCapacity = Coord::kNValid;

  class const_iterator {
  public:
    using iterator_category = std::forward_iterator_tag;
    using value_type = Coord;
    using difference_type = std::ptrdiff_t;
    using pointer = const Coord*;
    using reference = Coord;

    const_iterator() = default;

    Coord operator*() const {
      assert(idx_ < kCapacity && "dereferencing CoordSet::end()");
      return Coord::from_index(static_cast<uint16_t>(idx_)).value();
    }

    const_iterator& operator++() {
      if (set_ != nullptr) {
        ++idx_;
        skip();
      }
      return *this;
    }

    bool operator==(const const_iterator& other) const {
      return idx_ == other.idx_;
    }

    bool operator!=(const const_iterator& other) const {
      return !(*this == other);
    }

  private:
    friend class CoordSet;

    const_iterator(const CoordSet* set, std::size_t idx)
        : set_(set), idx_(idx) {}

    // Advances idx_ to the next set bit, or to kCapacity past the end.
    void skip() {
      while (set_ != nullptr && idx_ < kCapacity && !set_->bits_.test(idx_)) {
        ++idx_;
      }
    }

    const CoordSet* set_ = nullptr;
    std::size_t idx_ = 0;
  };

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

  // Returns the coordinate when present, mirroring the Rust get.
  std::optional<Coord> get(Coord coord) const {
    return contains(coord) ? std::optional<Coord>(coord) : std::nullopt;
  }

  // Removes and returns the coordinate when present, mirroring the Rust
  // take.
  std::optional<Coord> take(Coord coord) {
    return remove(coord) ? std::optional<Coord>(coord) : std::nullopt;
  }

  // Retains only the coordinates satisfying the predicate, mirroring the
  // Rust retain.
  template <typename F>
  void retain(F&& predicate) {
    for (int index = 0; index < kCapacity; ++index) {
      if (bits_.test(index)) {
        const Coord coord =
            Coord::from_index(static_cast<uint16_t>(index)).value();
        if (!predicate(coord)) bits_.reset(index);
      }
    }
  }

  // Iterates over the present coordinates in index order.
  const_iterator begin() const {
    const_iterator it(this, 0);
    it.skip();
    return it;
  }

  const_iterator end() const { return const_iterator(this, kCapacity); }

  bool operator==(const CoordSet& other) const { return bits_ == other.bits_; }

  bool operator!=(const CoordSet& other) const { return !(*this == other); }

private:
  std::bitset<kCapacity> bits_;
};

}  // namespace tagma
