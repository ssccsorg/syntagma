#pragma once

// CoordMapN: a fixed N-byte-key hashless map over the CoordSpaceN tree.
// Mirrors the Rust CoordMapN<N> in sw/rust/map/src/coord_map_n.rs. The
// CoordMap and CoordMapKey trait surfaces map to the method set below
// (insert/get/remove/contains_key via &str and via CoordKey<N>).

#include "tagma_map/coord_key.h"

#include "tagma_core/coord_space_n.h"

#include <cstddef>
#include <optional>
#include <string>
#include <vector>

namespace tagma_map {

template <int N>
class CoordMapN {
public:
  using Value = std::vector<uint8_t>;

  CoordMapN() = default;

  // The number of stored entries.
  std::size_t len() const { return len_; }

  bool is_empty() const { return len_ == 0; }

  // Removes all entries.
  void clear() {
    space_.clear();
    len_ = 0;
  }

  // ── &str-key API ────────────────────────────────────────────────────

  // Inserts a key-value pair; returns the previous value when present.
  // Throws std::invalid_argument when key.size() != N (mirrors the Rust
  // panic via CoordKey::from).
  std::optional<Value> insert(const std::string& key, Value value) {
    return insert_by_coordkey(CoordKey<N>::from_string(key),
                              std::move(value));
  }

  // Retrieves a value by key. Returns nullopt when the key length is not N
  // or the key is absent, mirroring the Rust reference (CoordMap::get
  // guards on key.len() != N).
  std::optional<Value> get(const std::string& key) const {
    if (key.size() != static_cast<std::size_t>(N)) return std::nullopt;
    return get_by_coordkey(CoordKey<N>::from_string(key));
  }

  // Removes a key-value pair; returns the value when present. Returns
  // nullopt when the key length is not N, mirroring the Rust reference.
  std::optional<Value> remove(const std::string& key) {
    if (key.size() != static_cast<std::size_t>(N)) return std::nullopt;
    return remove_by_coordkey(CoordKey<N>::from_string(key));
  }

  bool contains_key(const std::string& key) const {
    return get(key).has_value();
  }

  // ── CoordKey API ────────────────────────────────────────────────────

  std::optional<Value> insert_by_coordkey(const CoordKey<N>& key,
                                          Value value) {
    const auto path = key.to_coord_path();
    std::optional<Value> previous = space_.place_path(path, std::move(value));
    if (!previous) len_ += 1;
    return previous;
  }

  std::optional<Value> get_by_coordkey(const CoordKey<N>& key) const {
    const auto path = key.to_coord_path();
    const Value* value = space_.at_path(path);
    return value != nullptr ? std::optional<Value>(*value) : std::nullopt;
  }

  std::optional<Value> remove_by_coordkey(const CoordKey<N>& key) {
    const auto path = key.to_coord_path();
    std::optional<Value> value = space_.vacate_path(path);
    if (value) len_ -= 1;
    return value;
  }

  bool contains_key_by_coordkey(const CoordKey<N>& key) const {
    return get_by_coordkey(key).has_value();
  }

  // Looks up a value by a CoordPath, converting internally. Used by the
  // spatial queries in coord_cube_map.h.
  std::optional<Value> get_by_coord_path(
      const tagma::CoordPath<N>& path) const {
    const Value* value = space_.at_path(path);
    return value != nullptr ? std::optional<Value>(*value) : std::nullopt;
  }

  // All (key bytes, value) pairs in ascending coordinate order. The value
  // pointers stay valid until the next mutation of the store. Mirrors the
  // Rust CoordMapN::iter.
  std::vector<std::pair<std::array<uint8_t, N>, const Value*>> iter() const {
    std::vector<std::pair<std::array<uint8_t, N>, const Value*>> out;
    for (const auto& entry : space_.entries()) {
      std::array<uint8_t, N> key{};
      for (int i = 0; i < N; ++i) {
        key[i] = static_cast<uint8_t>(entry.first.coords()[i].index());
      }
      out.emplace_back(key, entry.second);
    }
    return out;
  }

private:
  tagma::CoordSpaceN<N, Value> space_;
  std::size_t len_ = 0;
};

}  // namespace tagma_map
