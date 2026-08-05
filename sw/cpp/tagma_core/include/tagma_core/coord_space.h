#pragma once

// CoordSpace: a direct-address table indexed by coordinate. One slot per
// valid coordinate, so placement and lookup are O(1) with no hashing and
// no collisions. Mirrors the Rust CoordSpace<V> in sw/rust/core.

#include "tagma_core/coord.h"

#include <array>
#include <cstddef>
#include <optional>

namespace tagma {

template <typename V>
class CoordSpace {
public:
  static constexpr int kCapacity = Coord::kNValid;

  CoordSpace() = default;

  // The number of entries.
  std::size_t len() const { return len_; }

  bool is_empty() const { return len_ == 0; }

  // The maximum capacity, always 11,172.
  static constexpr int capacity() { return kCapacity; }

  // A pointer to the value at coord, or nullptr when absent.
  const V* at(Coord coord) const {
    const std::optional<V>& slot = slots_[coord.index()];
    return slot ? &*slot : nullptr;
  }

  V* at_mut(Coord coord) {
    std::optional<V>& slot = slots_[coord.index()];
    return slot ? &*slot : nullptr;
  }

  // True when the space holds an entry for coord.
  bool occupied(Coord coord) const { return slots_[coord.index()].has_value(); }

  // Places value at coord, returning the previous value when present.
  std::optional<V> place(Coord coord, V value) {
    std::optional<V>& slot = slots_[coord.index()];
    std::optional<V> previous = std::move(slot);
    slot = std::move(value);
    if (!previous) len_ += 1;
    return previous;
  }

  // Removes and returns the value at coord, when present.
  std::optional<V> vacate(Coord coord) {
    std::optional<V>& slot = slots_[coord.index()];
    std::optional<V> previous = std::move(slot);
    slot.reset();
    if (previous) len_ -= 1;
    return previous;
  }

  // Removes all entries.
  void clear() {
    slots_.fill(std::nullopt);
    len_ = 0;
  }

private:
  std::array<std::optional<V>, kCapacity> slots_;
  std::size_t len_ = 0;
};

}  // namespace tagma
