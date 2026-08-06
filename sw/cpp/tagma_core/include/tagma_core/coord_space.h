#pragma once

// CoordSpace: a direct-address table indexed by coordinate. One slot per
// valid coordinate, so placement and lookup are O(1) with no hashing and
// no collisions. Mirrors the Rust CoordSpace<V> in sw/rust/core.

#include "tagma_core/coord.h"
#include "tagma_core/coord_path.h"

#include <array>
#include <cassert>
#include <cstddef>
#include <optional>
#include <utility>

namespace tagma {

template <typename V>
class CoordSpace {
public:
  static constexpr int kCapacity = Coord::kNValid;

  class const_iterator {
  public:
    using iterator_category = std::forward_iterator_tag;
    using value_type = std::pair<Coord, const V&>;
    using difference_type = std::ptrdiff_t;
    using pointer = const value_type*;
    using reference = value_type;

    const_iterator() = default;

    value_type operator*() const {
      assert(idx_ < kCapacity && "dereferencing CoordSpace::end()");
      return {Coord::from_index(static_cast<uint16_t>(idx_)).value(),
              *space_->slots_[idx_]};
    }

    const_iterator& operator++() {
      advance();
      return *this;
    }

    bool operator==(const const_iterator& other) const {
      return idx_ == other.idx_;
    }

    bool operator!=(const const_iterator& other) const {
      return !(*this == other);
    }

  private:
    friend class CoordSpace;

    const_iterator(const CoordSpace* space, std::size_t idx)
        : space_(space), idx_(idx) {}

    // Advances idx_ to the next occupied slot, or to kCapacity past the
    // end.
    void advance() {
      if (space_ == nullptr) return;
      do {
        ++idx_;
      } while (idx_ < kCapacity && !space_->slots_[idx_].has_value());
    }

    const CoordSpace* space_ = nullptr;
    std::size_t idx_ = 0;
  };

  // Entry API mirroring the Rust FlatEntry: or_insert inserts when the
  // slot is empty and returns a reference to the value.
  class Entry {
  public:
    Entry(CoordSpace* space, Coord coord) : space_(space), coord_(coord) {}

    V& or_insert(V value) {
      std::optional<V>& slot = space_->slots_[coord_.index()];
      if (!slot.has_value()) {
        slot = std::move(value);
        space_->len_ += 1;
      }
      return *slot;
    }

    template <typename F>
    V& or_insert_with(F&& factory) {
      std::optional<V>& slot = space_->slots_[coord_.index()];
      if (!slot.has_value()) {
        slot = factory();
        space_->len_ += 1;
      }
      return *slot;
    }

  private:
    CoordSpace* space_;
    Coord coord_;
  };

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

  // Single-character path access, mirroring the Rust at_path.
  const V* at_path(const CoordPath<1>& path) const { return at(path.coords()[0]); }

  std::optional<V> place_path(const CoordPath<1>& path, V value) {
    return place(path.coords()[0], std::move(value));
  }

  std::optional<V> vacate_path(const CoordPath<1>& path) {
    return vacate(path.coords()[0]);
  }

  // Entry API, mirroring the Rust entry().or_insert().
  Entry entry(Coord coord) { return Entry(this, coord); }

  // Iterates over the occupied (coord, value) pairs in index order.
  const_iterator begin() const {
    const_iterator it(this, 0);
    if (!slots_[0].has_value()) it.advance();
    return it;
  }

  const_iterator end() const { return const_iterator(this, kCapacity); }

  // Retains only the entries satisfying the predicate, mirroring the Rust
  // retain.
  template <typename F>
  void retain(F&& predicate) {
    for (std::size_t index = 0; index < kCapacity; ++index) {
      std::optional<V>& slot = slots_[index];
      if (slot.has_value()) {
        const Coord coord =
            Coord::from_index(static_cast<uint16_t>(index)).value();
        if (!predicate(coord, *slot)) {
          slot.reset();
          len_ -= 1;
        }
      }
    }
  }

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
  // One slot per valid coordinate, 11,172 entries inline. For V = int this
  // is about 89 KB per CoordSpace; CoordSpaceN<1, V> embeds the same leaf.
  // Mirroring the Rust reference, prefer heap allocation when the object is
  // stored by value or on the stack.
  std::array<std::optional<V>, kCapacity> slots_;
  std::size_t len_ = 0;
};

}  // namespace tagma
