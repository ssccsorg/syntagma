#pragma once

// CoordSpaceN: an N-level direct-address tree indexed by CoordPath<N>.
// Levels allocate lazily: only paths that are actually placed consume
// nodes. Navigation is closed-form through each level, no hashing and no
// collision resolution. Mirrors the Rust CoordSpaceN<N, V> in
// sw/rust/core.

#include "tagma_core/coord.h"
#include "tagma_core/coord_path.h"

#include <array>
#include <cstddef>
#include <memory>
#include <optional>
#include <type_traits>
#include <utility>
#include <vector>

namespace tagma {

// Node storage for the lazy tree. SpaceNode<0> is a leaf holding one slot
// per valid coordinate; SpaceNode<D> (D > 0) is a branch holding lazily
// allocated children. Mirrors the Rust Node enum in coord_space_n.rs.
template <int D, typename V>
struct SpaceNode {
  static_assert(D > 0, "SpaceNode depth must be positive");
  std::array<std::unique_ptr<SpaceNode<D - 1, V>>, Coord::kNValid> children;
};

template <typename V>
struct SpaceNode<0, V> {
  std::array<std::optional<V>, Coord::kNValid> slots;
};

template <int N, typename V>
class CoordSpaceN {
public:
  static_assert(N >= 1, "CoordSpaceN depth N must be at least 1");

  static constexpr int kCapacity = Coord::kNValid;

  using Leaf = SpaceNode<0, V>;
  using Node = SpaceNode<N - 1, V>;

  CoordSpaceN() = default;

  // The number of entries.
  std::size_t len() const { return len_; }

  bool is_empty() const { return len_ == 0; }

  // Maximum capacity: 11,172 for N=1, nullopt for N>1 (the tree grows
  // dynamically).
  std::optional<std::size_t> capacity() const {
    if constexpr (N == 1) {
      return static_cast<std::size_t>(kCapacity);
    } else {
      return std::nullopt;
    }
  }

  // ── General N: path-based access ───────────────────────────────────

  // A pointer to the value at path, or nullptr when absent.
  const V* at_path(const CoordPath<N>& path) const {
    const Leaf* leaf = descend<N - 1>(root_, path.coords(), 0);
    if (leaf == nullptr) return nullptr;
    const std::optional<V>& slot = leaf->slots[path.coords()[N - 1].index()];
    return slot ? &*slot : nullptr;
  }

  V* at_path_mut(const CoordPath<N>& path) {
    Leaf* leaf = descend_mut<N - 1>(root_, path.coords(), 0);
    if (leaf == nullptr) return nullptr;
    std::optional<V>& slot = leaf->slots[path.coords()[N - 1].index()];
    return slot ? &*slot : nullptr;
  }

  // Places value at path, returning the previous value when present.
  std::optional<V> place_path(const CoordPath<N>& path, V value) {
    Leaf* leaf = descend_create<N - 1>(root_, path.coords(), 0);
    std::optional<V>& slot = leaf->slots[path.coords()[N - 1].index()];
    std::optional<V> previous = std::move(slot);
    slot = std::move(value);
    if (!previous) len_ += 1;
    return previous;
  }

  // Removes the value at path, returning it when present.
  std::optional<V> vacate_path(const CoordPath<N>& path) {
    Leaf* leaf = descend_mut<N - 1>(root_, path.coords(), 0);
    if (leaf == nullptr) return std::nullopt;
    std::optional<V>& slot = leaf->slots[path.coords()[N - 1].index()];
    std::optional<V> previous = std::move(slot);
    slot.reset();
    if (previous) len_ -= 1;
    return previous;
  }

  // Removes all entries, preserving the empty tree.
  void clear() {
    root_ = Node{};
    len_ = 0;
  }

  // All present paths in depth-first coordinate-ascending order. The C++
  // port represents the Rust lazy TreeIter as an eager collection.
  std::vector<CoordPath<N>> paths() const {
    std::vector<CoordPath<N>> out;
    std::array<Coord, N> prefix{};
    collect_paths<N - 1>(root_, prefix, 0, out);
    return out;
  }

  // All (path, value) pairs in depth-first coordinate-ascending order.
  // The value pointers stay valid until the next mutation of the space.
  std::vector<std::pair<CoordPath<N>, const V*>> entries() const {
    std::vector<std::pair<CoordPath<N>, const V*>> out;
    std::array<Coord, N> prefix{};
    collect_entries<N - 1>(root_, prefix, 0, out);
    return out;
  }

  // ── N == 1: single-coordinate access ───────────────────────────────

  template <bool Enable = (N == 1), typename = std::enable_if_t<Enable>>
  const V* at(Coord coord) const {
    const std::optional<V>& slot = root_.slots[coord.index()];
    return slot ? &*slot : nullptr;
  }

  template <bool Enable = (N == 1), typename = std::enable_if_t<Enable>>
  V* at_mut(Coord coord) {
    std::optional<V>& slot = root_.slots[coord.index()];
    return slot ? &*slot : nullptr;
  }

  template <bool Enable = (N == 1), typename = std::enable_if_t<Enable>>
  bool occupied(Coord coord) const {
    return root_.slots[coord.index()].has_value();
  }

  template <bool Enable = (N == 1), typename = std::enable_if_t<Enable>>
  std::optional<V> place(Coord coord, V value) {
    std::optional<V>& slot = root_.slots[coord.index()];
    std::optional<V> previous = std::move(slot);
    slot = std::move(value);
    if (!previous) len_ += 1;
    return previous;
  }

  template <bool Enable = (N == 1), typename = std::enable_if_t<Enable>>
  std::optional<V> vacate(Coord coord) {
    std::optional<V>& slot = root_.slots[coord.index()];
    std::optional<V> previous = std::move(slot);
    slot.reset();
    if (previous) len_ -= 1;
    return previous;
  }

  // Entry API mirroring the Rust Entry for N=1: or_insert inserts when
  // the slot is empty and returns a reference to the value.
  class Entry {
  public:
    Entry(CoordSpaceN* space, Coord coord) : space_(space), coord_(coord) {}

    V& or_insert(V value) {
      std::optional<V>& slot = space_->root_.slots[coord_.index()];
      if (!slot.has_value()) {
        slot = std::move(value);
        space_->len_ += 1;
      }
      return *slot;
    }

    template <typename F>
    V& or_insert_with(F&& factory) {
      std::optional<V>& slot = space_->root_.slots[coord_.index()];
      if (!slot.has_value()) {
        slot = factory();
        space_->len_ += 1;
      }
      return *slot;
    }

  private:
    CoordSpaceN* space_;
    Coord coord_;
  };

  template <bool Enable = (N == 1), typename = std::enable_if_t<Enable>>
  Entry entry(Coord coord) {
    return Entry(this, coord);
  }

private:
  // Descends through branch levels, returning the leaf for the path, or
  // nullptr when a level is missing.
  template <int D>
  static const Leaf* descend(const SpaceNode<D, V>& node,
                             const std::array<Coord, N>& coords, int level) {
    if constexpr (D == 0) {
      return &node;
    } else {
      const auto& child = node.children[coords[level].index()];
      return child ? descend<D - 1>(*child, coords, level + 1) : nullptr;
    }
  }

  template <int D>
  static Leaf* descend_mut(SpaceNode<D, V>& node,
                           const std::array<Coord, N>& coords, int level) {
    if constexpr (D == 0) {
      return &node;
    } else {
      auto& child = node.children[coords[level].index()];
      return child ? descend_mut<D - 1>(*child, coords, level + 1) : nullptr;
    }
  }

  // Descends, creating missing levels on the way (used by place_path).
  template <int D>
  static Leaf* descend_create(SpaceNode<D, V>& node,
                              const std::array<Coord, N>& coords, int level) {
    if constexpr (D == 0) {
      return &node;
    } else {
      auto& child = node.children[coords[level].index()];
      if (!child) child = std::make_unique<SpaceNode<D - 1, V>>();
      return descend_create<D - 1>(*child, coords, level + 1);
    }
  }

  // Collects present paths in depth-first coordinate-ascending order.
  template <int D>
  static void collect_paths(const SpaceNode<D, V>& node,
                            std::array<Coord, N>& prefix, int level,
                            std::vector<CoordPath<N>>& out) {
    if constexpr (D == 0) {
      for (int i = 0; i < kCapacity; ++i) {
        if (node.slots[i].has_value()) {
          prefix[level] = Coord::from_index(static_cast<uint16_t>(i)).value();
          out.push_back(CoordPath<N>::from_array(prefix));
        }
      }
    } else {
      for (int i = 0; i < kCapacity; ++i) {
        const auto& child = node.children[i];
        if (child) {
          prefix[level] = Coord::from_index(static_cast<uint16_t>(i)).value();
          collect_paths<D - 1>(*child, prefix, level + 1, out);
        }
      }
    }
  }

  // Collects present (path, value) pairs in depth-first
  // coordinate-ascending order.
  template <int D>
  static void collect_entries(const SpaceNode<D, V>& node,
                              std::array<Coord, N>& prefix, int level,
                              std::vector<std::pair<CoordPath<N>, const V*>>&
                                  out) {
    if constexpr (D == 0) {
      for (int i = 0; i < kCapacity; ++i) {
        if (node.slots[i].has_value()) {
          prefix[level] = Coord::from_index(static_cast<uint16_t>(i)).value();
          out.emplace_back(CoordPath<N>::from_array(prefix), &*node.slots[i]);
        }
      }
    } else {
      for (int i = 0; i < kCapacity; ++i) {
        const auto& child = node.children[i];
        if (child) {
          prefix[level] = Coord::from_index(static_cast<uint16_t>(i)).value();
          collect_entries<D - 1>(*child, prefix, level + 1, out);
        }
      }
    }
  }

  Node root_;
  std::size_t len_ = 0;
};

// Type aliases mirroring the Rust aliases.
template <typename V>
using CoordSpaceN1 = CoordSpaceN<1, V>;
template <typename V>
using CoordSpaceN2 = CoordSpaceN<2, V>;
template <typename V>
using CoordSpaceN3 = CoordSpaceN<3, V>;
template <typename V>
using CoordSpaceN6 = CoordSpaceN<6, V>;
template <typename V>
using CoordSpaceN12 = CoordSpaceN<12, V>;
template <typename V>
using CoordSpaceN19 = CoordSpaceN<19, V>;

}  // namespace tagma
