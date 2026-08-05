#pragma once

// DynCoordSpace: a collision-free space indexed by a slice of Coords with
// dynamic depth. Each level is a fixed 11,172-slot array indexed directly
// by Coord; no hashing, no collisions, regardless of depth. Depth is
// determined at runtime by the path length; memory allocates lazily.
// Mirrors the Rust DynCoordSpace<V> in sw/rust/core.

#include "tagma_core/coord.h"

#include <array>
#include <cstddef>
#include <memory>
#include <optional>
#include <stdexcept>
#include <utility>
#include <vector>

namespace tagma {

template <typename V>
class DynCoordSpace {
public:
  DynCoordSpace() = default;

  // A pointer to the value at path, or nullptr when absent or when path is
  // empty. O(path.size()) with one array access per Coord.
  const V* at(const std::vector<Coord>& path) const {
    if (path.empty()) return nullptr;
    const DynCoordSpace* node = this;
    for (std::size_t i = 0; i < path.size(); ++i) {
      const std::size_t index = path[i].index();
      const Slot& slot = node->slots_[index];
      const bool last = (i == path.size() - 1);
      switch (slot.kind) {
        case Slot::Kind::kEmpty:
          return nullptr;
        case Slot::Kind::kLeaf:
        case Slot::Kind::kBoth:
          if (last) return &*slot.value;
          if (slot.child) {
            node = slot.child.get();
            continue;
          }
          return nullptr;
        case Slot::Kind::kNode:
          if (last) return nullptr;  // a node holds no value at its own level
          node = slot.child.get();
          continue;
      }
    }
    return nullptr;
  }

  // Inserts a value at path, creating intermediate nodes as needed. Returns
  // the previous value when the exact path already existed. Throws
  // std::invalid_argument for an empty path, mirroring the Rust panic.
  std::optional<V> place(const std::vector<Coord>& path, V value) {
    if (path.empty()) {
      throw std::invalid_argument("DynCoordSpace: path must not be empty");
    }
    return insert_rec(path, 0, std::move(value));
  }

  // Removes the value at path, returning it when present. Returns nullopt
  // for an empty path.
  std::optional<V> vacate(const std::vector<Coord>& path) {
    if (path.empty()) return std::nullopt;
    return remove_rec(path, 0);
  }

  // Removes all entries.
  void clear() {
    for (Slot& slot : slots_) slot = Slot{};
  }

  // The number of entries across all depths, O(entries).
  std::size_t entry_count() const { return count_rec(); }

  // All (path, value) pairs in depth-first coordinate-ascending order. The
  // C++ port represents the Rust lazy DynIter as an eager collection.
  std::vector<std::pair<std::vector<Coord>, const V*>> entries() const {
    std::vector<std::pair<std::vector<Coord>, const V*>> out;
    std::vector<Coord> path;
    collect(path, out);
    return out;
  }

private:
  struct Slot {
    enum class Kind { kEmpty, kLeaf, kNode, kBoth };
    Kind kind = Kind::kEmpty;
    std::optional<V> value;
    std::unique_ptr<DynCoordSpace> child;
  };

  std::optional<V> insert_rec(const std::vector<Coord>& path,
                              std::size_t depth, V value) {
    const std::size_t index = path[depth].index();
    Slot& slot = slots_[index];
    if (depth == path.size() - 1) {
      switch (slot.kind) {
        case Slot::Kind::kLeaf:
        case Slot::Kind::kBoth: {
          std::optional<V> previous = std::move(slot.value);
          slot.value = std::move(value);
          return previous;
        }
        case Slot::Kind::kNode: {
          slot = Slot{};
          slot.kind = Slot::Kind::kLeaf;
          slot.value = std::move(value);
          return std::nullopt;
        }
        default: {
          slot.kind = Slot::Kind::kLeaf;
          slot.value = std::move(value);
          return std::nullopt;
        }
      }
    }
    if (slot.kind == Slot::Kind::kEmpty) {
      slot.kind = Slot::Kind::kNode;
      slot.child = std::make_unique<DynCoordSpace>();
    } else if (slot.kind == Slot::Kind::kLeaf) {
      // Promote a leaf into a Both, preserving the existing value.
      slot.kind = Slot::Kind::kBoth;
      slot.child = std::make_unique<DynCoordSpace>();
    }
    return slot.child->insert_rec(path, depth + 1, std::move(value));
  }

  std::optional<V> remove_rec(const std::vector<Coord>& path,
                              std::size_t depth) {
    const std::size_t index = path[depth].index();
    Slot& slot = slots_[index];
    if (depth == path.size() - 1) {
      switch (slot.kind) {
        case Slot::Kind::kLeaf: {
          std::optional<V> value = std::move(slot.value);
          slot = Slot{};
          return value;
        }
        case Slot::Kind::kBoth: {
          std::optional<V> value = std::move(slot.value);
          slot.kind = Slot::Kind::kNode;  // preserve deeper paths
          slot.value.reset();
          return value;
        }
        default:
          return std::nullopt;
      }
    }
    switch (slot.kind) {
      case Slot::Kind::kNode:
      case Slot::Kind::kBoth:
        return slot.child->remove_rec(path, depth + 1);
      default:
        return std::nullopt;
    }
  }

  std::size_t count_rec() const {
    std::size_t count = 0;
    for (const Slot& slot : slots_) {
      switch (slot.kind) {
        case Slot::Kind::kLeaf:
          count += 1;
          break;
        case Slot::Kind::kNode:
          count += slot.child->count_rec();
          break;
        case Slot::Kind::kBoth:
          count += 1 + slot.child->count_rec();
          break;
        default:
          break;
      }
    }
    return count;
  }

  void collect(
      std::vector<Coord>& path,
      std::vector<std::pair<std::vector<Coord>, const V*>>& out) const {
    for (int i = 0; i < Coord::kNValid; ++i) {
      const Slot& slot = slots_[i];
      if (slot.kind == Slot::Kind::kEmpty) continue;
      const Coord coord = Coord::from_index(static_cast<uint16_t>(i)).value();
      switch (slot.kind) {
        case Slot::Kind::kLeaf:
          path.push_back(coord);
          out.emplace_back(path, &*slot.value);
          path.pop_back();
          break;
        case Slot::Kind::kNode:
          path.push_back(coord);
          slot.child->collect(path, out);
          path.pop_back();
          break;
        case Slot::Kind::kBoth:
          path.push_back(coord);
          out.emplace_back(path, &*slot.value);
          slot.child->collect(path, out);
          path.pop_back();
          break;
        default:
          break;
      }
    }
  }

  std::array<Slot, Coord::kNValid> slots_;
};

}  // namespace tagma
