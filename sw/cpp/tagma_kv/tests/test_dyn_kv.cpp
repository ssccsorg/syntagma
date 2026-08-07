// Verifies DynCoordSpace (variable-depth tree) and DynCoordKV (dynamic
// string-key store). The checks mirror the Rust dyn_coord_space and
// dyn_coord_kv semantics.

#include <tagma_kv/dyn_coord_kv.h>
#include <tagma_kv/coord_gen.h>

#include <tagma_core/coord.h>
#include <tagma_core/dyn_coord_space.h>

#include <cstdio>
#include <cstdint>
#include <stdexcept>
#include <string>
#include <vector>

namespace {

int failures = 0;

void check(bool condition, const char* message) {
  if (!condition) {
    std::fprintf(stderr, "FAIL: %s\n", message);
    failures += 1;
  }
}

tagma::Coord coord(uint16_t index) {
  return tagma::Coord::from_index(index).value();
}

std::vector<uint8_t> bytes(const std::string& s) {
  return std::vector<uint8_t>(s.begin(), s.end());
}

void test_dyn_coord_space() {
  using tagma::DynCoordSpace;
  DynCoordSpace<int> space;
  check(space.entry_count() == 0, "dyn empty");

  // Prefix paths share nodes: "a" and "ab" coexist (Leaf vs Both).
  const std::vector<tagma::Coord> path_a = {coord('a')};
  const std::vector<tagma::Coord> path_ab = {coord('a'), coord('b')};
  const std::vector<tagma::Coord> path_ac = {coord('a'), coord('c')};

  check(space.place(path_a, 1) == std::nullopt, "dyn place a");
  check(space.place(path_ab, 2) == std::nullopt, "dyn place ab");
  check(space.at(path_a) != nullptr && *space.at(path_a) == 1, "dyn at a");
  check(space.at(path_ab) != nullptr && *space.at(path_ab) == 2, "dyn at ab");
  check(space.at(path_ac) == nullptr, "dyn at ac absent");
  check(space.entry_count() == 2, "dyn entry count");

  check(space.place(path_ab, 20) == std::optional<int>(2), "dyn replace");
  check(space.entry_count() == 2, "dyn count after replace");

  check(space.vacate(path_ab) == std::optional<int>(20), "dyn vacate ab");
  check(space.at(path_ab) == nullptr, "dyn ab gone");
  check(space.at(path_a) != nullptr, "dyn a preserved after vacate ab");

  // A deeper path after vacate: "a" is now a Leaf, "ac" promotes to Both.
  check(space.place(path_ac, 3) == std::nullopt, "dyn place ac after vacate");
  check(space.at(path_a) != nullptr && *space.at(path_a) == 1,
        "dyn a preserved with child");
  check(space.at(path_ac) != nullptr && *space.at(path_ac) == 3,
        "dyn ac present");

  check(space.entries().size() == 2, "dyn entries size");

  bool threw = false;
  try {
    space.place({}, 5);
  } catch (const std::invalid_argument&) {
    threw = true;
  }
  check(threw, "dyn empty path throws");

  space.clear();
  check(space.entry_count() == 0, "dyn clear");
}

void test_dyn_coord_kv() {
  using tagma_kv::DynCoordKV;
  DynCoordKV kv;
  check(kv.is_empty() && kv.len() == 0, "dynkv initially empty");

  check(kv.insert("hello", bytes("world")) == std::nullopt, "dynkv insert");
  check(kv.len() == 1, "dynkv length");
  check(kv.get("hello") == std::optional<std::vector<uint8_t>>(bytes("world")),
        "dynkv get");
  check(kv.contains_key("hello"), "dynkv contains");

  check(kv.insert("hello", bytes("earth")) ==
            std::optional<std::vector<uint8_t>>(bytes("world")),
        "dynkv replace returns previous");
  check(kv.len() == 1, "dynkv length after replace");

  check(kv.insert("한글", bytes("v")) == std::nullopt, "dynkv unicode key");
  check(kv.get("한글") == std::optional<std::vector<uint8_t>>(bytes("v")),
        "dynkv unicode get");
  check(kv.len() == 2, "dynkv length with unicode");

  check(kv.insert("", bytes("x")) == std::nullopt, "dynkv empty key rejected");
  check(kv.get("") == std::nullopt, "dynkv empty get");

  check(kv.remove("hello") ==
            std::optional<std::vector<uint8_t>>(bytes("earth")),
        "dynkv remove");
  check(kv.get("hello") == std::nullopt, "dynkv removed get");
  check(kv.len() == 1, "dynkv length after remove");

  kv.clear();
  check(kv.is_empty() && kv.len() == 0, "dynkv clear");
}

void test_dyn_iter() {
  using tagma_kv::DynCoordKV;
  const DynCoordKV empty;
  check(empty.iter().empty(), "dyn iter empty");

  DynCoordKV kv;
  kv.insert("abc", bytes("123"));
  kv.insert("def", bytes("456"));
  const auto entries = kv.iter();
  check(entries.size() == 2, "dyn iter size");
  check(entries[0].first == "abc" && *entries[0].second == bytes("123"),
        "dyn iter first in ascending order");
  check(entries[1].first == "def" && *entries[1].second == bytes("456"),
        "dyn iter second in ascending order");
}

void test_dyn_long_key_roundtrip() {
  // 64-byte key: the ByteWise path length equals the key size and lookup
  // is O(key length). Mirrors dyn_roundtrip_large_key.
  std::string long_key;
  long_key.reserve(64);
  for (int i = 0; i < 64; ++i) {
    long_key.push_back(static_cast<char>(i));
  }
  tagma_kv::DynCoordKV kv;
  check(kv.insert(long_key, bytes("v")) == std::nullopt,
        "dyn long key insert");
  check(kv.get(long_key) ==
            std::optional<std::vector<uint8_t>>(bytes("v")),
        "dyn long key get");
  check(kv.len() == 1, "dyn long key len");
}

}  // namespace

int main() {
  test_dyn_coord_space();
  test_dyn_coord_kv();
  test_dyn_iter();
  test_dyn_long_key_roundtrip();

  if (failures != 0) {
    std::fprintf(stderr, "%d check(s) failed\n", failures);
    return 1;
  }
  std::printf("tagma_kv dyn: all checks passed\n");
  return 0;
}
