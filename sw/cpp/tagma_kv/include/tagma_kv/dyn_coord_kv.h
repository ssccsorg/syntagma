#pragma once

// DynCoordKV: a hash-free, collision-free string KV store backed by
// DynCoordSpace with ByteWise coordinate generation. Supports any
// non-empty string key; lookup is O(key length). Mirrors the Rust
// DynCoordKV in sw/rust/kv/src/dyn_coord_kv.rs.

#include "tagma_kv/coord_gen.h"

#include "tagma_core/coord_path.h"
#include "tagma_core/dyn_coord_space.h"

#include <cstddef>
#include <optional>
#include <string>
#include <utility>
#include <vector>

namespace tagma_kv {

class DynCoordKV {
public:
  using Value = std::vector<uint8_t>;

  DynCoordKV() = default;

  // The number of stored entries.
  std::size_t len() const { return len_; }

  bool is_empty() const { return len_ == 0; }

  // Removes all entries.
  void clear() {
    space_.clear();
    len_ = 0;
  }

  // Inserts a key-value pair; returns the previous value when present.
  // Empty keys are rejected with nullopt, mirroring the Rust behavior.
  std::optional<Value> insert(const std::string& key, Value value) {
    if (key.empty()) return std::nullopt;
    const auto path = string_to_coord_path(key);
    std::optional<Value> previous = space_.place(*path, std::move(value));
    if (!previous) len_ += 1;
    return previous;
  }

  // Retrieves a value by key. Returns nullopt for empty keys or absent
  // keys.
  std::optional<Value> get(const std::string& key) const {
    if (key.empty()) return std::nullopt;
    const auto path = string_to_coord_path(key);
    const Value* value = space_.at(*path);
    return value != nullptr ? std::optional<Value>(*value) : std::nullopt;
  }

  // Removes a key-value pair; returns the value when present.
  std::optional<Value> remove(const std::string& key) {
    if (key.empty()) return std::nullopt;
    const auto path = string_to_coord_path(key);
    std::optional<Value> value = space_.vacate(*path);
    if (value) len_ -= 1;
    return value;
  }

  bool contains_key(const std::string& key) const {
    return get(key).has_value();
  }

  // Looks up a value by a fixed-length CoordPath, converting internally.
  template <int N>
  std::optional<Value> get_by_coord_path(
      const tagma::CoordPath<N>& path) const {
    std::vector<tagma::Coord> coords(path.coords().begin(),
                                     path.coords().end());
    const Value* value = space_.at(coords);
    return value != nullptr ? std::optional<Value>(*value) : std::nullopt;
  }

private:
  tagma::DynCoordSpace<Value> space_;
  std::size_t len_ = 0;
};

}  // namespace tagma_kv
