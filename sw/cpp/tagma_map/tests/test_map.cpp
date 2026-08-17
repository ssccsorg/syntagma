// Verifies the tagma_map hashless string-key map: generation strategies,
// CoordKey, and CoordMapN. The checks mirror the Rust tagma-map coord_gen
// and coord_map_n tests.

#include <tagma_map/coord_gen.h>
#include <tagma_map/coord_key.h>
#include <tagma_map/coord_map_n.h>

#include <array>
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

std::vector<uint8_t> bytes(const std::string& s) {
  return std::vector<uint8_t>(s.begin(), s.end());
}

void test_bytewise() {
  const auto coords = tagma_map::ByteWise::generate("hi");
  check(coords.has_value() && coords->size() == 2, "bytewise path length");
  if (coords) {
    check((*coords)[0].index() == 'h' && (*coords)[1].index() == 'i',
          "bytewise indices");
  }
  check(!tagma_map::ByteWise::generate("").has_value(), "bytewise empty");
  check(tagma_map::ByteWise::is_injective(), "bytewise injective");
  check(!tagma_map::ByteWise::fixed_depth().has_value(), "bytewise dynamic");

  // Unicode: UTF-8 bytes map one-to-one.
  const auto korean = tagma_map::ByteWise::generate("가");
  check(korean.has_value() && korean->size() == 3, "bytewise utf8 bytes");
}

void test_charwise() {
  const auto coords = tagma_map::CharWise::generate("hi");
  check(coords.has_value() && coords->size() == 4, "charwise two per char");
  if (coords) {
    const uint32_t h = 'h';
    check((*coords)[0].index() == h / 11172 && (*coords)[1].index() == h % 11172,
          "charwise first char split");
  }
  const auto hangul = tagma_map::CharWise::generate("가");
  check(hangul.has_value() && hangul->size() == 2, "charwise hangul pair");
  check(!tagma_map::CharWise::generate("").has_value(), "charwise empty");
}

void test_prefix() {
  const auto coords = tagma_map::Prefix<4>::generate("hi");
  check(coords.has_value() && coords->size() == 4, "prefix length 4");
  if (coords) {
    check((*coords)[0].index() == 'h' && (*coords)[1].index() == 'i' &&
              (*coords)[2].index() == 0 && (*coords)[3].index() == 0,
          "prefix zero pad");
  }
  check(!tagma_map::Prefix<4>::is_injective(), "prefix lossy");
  check(tagma_map::Prefix<4>::fixed_depth() == std::optional<std::size_t>(4),
        "prefix fixed depth");
}

void test_bytefold() {
  const auto coords = tagma_map::ByteFold<2>::generate("ab");
  check(coords.has_value() && coords->size() == 2, "bytefold length");
  if (coords) {
    check((*coords)[0].index() == 'a' && (*coords)[1].index() == 'b',
          "bytefold accumulators");
  }
  const auto again = tagma_map::ByteFold<2>::generate("ab");
  check(again == coords, "bytefold deterministic");
  check(!tagma_map::ByteFold<2>::is_injective(), "bytefold lossy");
}

void test_string_to_coord_path() {
  const auto a = tagma_map::string_to_coord_path("key");
  const auto b = tagma_map::ByteWise::generate("key");
  check(a.has_value() && a == b, "string_to_coord_path matches bytewise");
  check(!tagma_map::string_to_coord_path("").has_value(), "empty rejected");
}

void test_coord_key() {
  using tagma_map::CoordKey;
  const CoordKey<2> key(std::array<uint8_t, 2>{{'h', 'i'}});
  check(key.as_bytes() == std::array<uint8_t, 2>{{'h', 'i'}}, "coord key bytes");
  const auto path = key.to_coord_path();
  check(path.coords()[0].index() == 'h' && path.coords()[1].index() == 'i',
        "coord key path");

  const CoordKey<2> from_str = CoordKey<2>::from_string("hi");
  check(from_str == key, "coord key from string");

  bool threw = false;
  try {
    CoordKey<2>::from_string("hello");
  } catch (const std::invalid_argument&) {
    threw = true;
  }
  check(threw, "coord key wrong length throws");
}

void test_coord_mapn() {
  using tagma_map::CoordMapN;
  CoordMapN<2> map;
  check(map.is_empty() && map.len() == 0, "kvn initially empty");

  check(map.insert("hi", bytes("v1")) == std::nullopt, "kvn first insert");
  check(map.len() == 1, "kvn length");
  check(map.contains_key("hi"), "kvn contains");
  check(map.get("hi") == std::optional<std::vector<uint8_t>>(bytes("v1")),
        "kvn get");
  check(map.get("no") == std::nullopt, "kvn get absent");

  check(map.insert("hi", bytes("v2")) ==
            std::optional<std::vector<uint8_t>>(bytes("v1")),
        "kvn replace returns previous");
  check(map.len() == 1, "kvn length after replace");

  check(map.remove("hi") == std::optional<std::vector<uint8_t>>(bytes("v2")),
        "kvn remove");
  check(map.is_empty(), "kvn empty after remove");
  check(map.get("hi") == std::nullopt, "kvn get after remove");

  // Wrong-length keys: get/remove return nullopt, insert throws, mirroring
  // the Rust reference (CoordMap::get/remove guard on length; insert panics
  // via CoordKey::from).
  check(map.get("x") == std::nullopt, "kvn wrong length get");
  check(map.remove("x") == std::nullopt, "kvn wrong length remove");
  bool threw = false;
  try {
    map.insert("x", bytes("v"));
  } catch (const std::invalid_argument&) {
    threw = true;
  }
  check(threw, "kvn wrong length insert throws");

  // CoordKey API.
  const tagma_map::CoordKey<2> key(std::array<uint8_t, 2>{{'a', 'b'}});
  check(map.insert_by_coordkey(key, bytes("v3")) == std::nullopt,
        "kvn insert by coordkey");
  check(map.contains_key_by_coordkey(key), "kvn contains by coordkey");
  check(map.get_by_coordkey(key) ==
            std::optional<std::vector<uint8_t>>(bytes("v3")),
        "kvn get by coordkey");

  map.clear();
  check(map.is_empty() && map.len() == 0, "kvn clear");
}

void test_mapn_iter() {
  using tagma_map::CoordMapN;
  const CoordMapN<2> empty;
  check(empty.iter().empty(), "kvn iter empty");

  CoordMapN<2> map;
  map.insert("aa", bytes("1"));
  map.insert("bb", bytes("2"));
  const auto entries = map.iter();
  check(entries.size() == 2, "kvn iter size");
  check(entries[0].first == std::array<uint8_t, 2>{{'a', 'a'}} &&
            *entries[0].second == bytes("1"),
        "kvn iter first in ascending order");
  check(entries[1].first == std::array<uint8_t, 2>{{'b', 'b'}} &&
            *entries[1].second == bytes("2"),
        "kvn iter second in ascending order");
}

void test_strategy_edge_cases() {
  // Empty keys are rejected by every strategy.
  check(!tagma_map::Prefix<2>::generate("").has_value(),
        "prefix empty rejected");
  check(!tagma_map::ByteFold<2>::generate("").has_value(),
        "bytefold empty rejected");

  // ByteFold: XOR is commutative within each accumulator; swapping
  // even-position bytes between two strings produces a collision.
  // Mirrors bytefold_collision_same_xor.
  const std::string key_a{'a', '\0', 'c'};
  const std::string key_b{'c', '\0', 'a'};
  const auto path_a = tagma_map::ByteFold<2>::generate(key_a);
  const auto path_b = tagma_map::ByteFold<2>::generate(key_b);
  check(path_a.has_value() && path_b.has_value() && path_a == path_b,
        "bytefold commutative xor collision");
  if (path_a) {
    check((*path_a)[0].index() == 2 && (*path_a)[1].index() == 0,
          "bytefold collision accumulators");
  }

  // ByteFold accumulator values: "abcd" folds to acc[0] = 'a'^'c',
  // acc[1] = 'b'^'d'. Mirrors bytefold_basic.
  const auto folded = tagma_map::ByteFold<2>::generate("abcd");
  check(folded.has_value() && folded->size() == 2, "bytefold path length");
  if (folded) {
    check((*folded)[0].index() == ('a' ^ 'c') &&
              (*folded)[1].index() == ('b' ^ 'd'),
          "bytefold accumulator values");
  }

  // Prefix truncation: keys sharing the first N bytes collide.
  const auto short_key = tagma_map::Prefix<2>::generate("ab");
  const auto long_key = tagma_map::Prefix<2>::generate("abcdef");
  check(short_key.has_value() && long_key.has_value() &&
            short_key == long_key,
        "prefix truncation collision");

  // Prefix<2> matches the first two ByteWise coords. Mirrors
  // prefix2_matches_bytewise_first_two.
  const auto prefixed = tagma_map::Prefix<2>::generate("abcd");
  const auto bytewise = tagma_map::ByteWise::generate("abcd");
  check(prefixed.has_value() && bytewise.has_value() &&
            bytewise->size() >= 2 && (*prefixed)[0] == (*bytewise)[0] &&
            (*prefixed)[1] == (*bytewise)[1],
        "prefix2 matches bytewise first two");
}

void test_coord_key_roundtrip() {
  using tagma_map::CoordKey;
  const CoordKey<3> key(std::array<uint8_t, 3>{{1, 255, 42}});
  const CoordKey<3> back = CoordKey<3>::from_coord_path(key.to_coord_path());
  check(back == key, "coord key roundtrip");

  // Distinct byte keys map to distinct paths.
  const CoordKey<2> k1(std::array<uint8_t, 2>{{0, 1}});
  const CoordKey<2> k2(std::array<uint8_t, 2>{{1, 0}});
  check(!(k1.to_coord_path() == k2.to_coord_path()), "coord key injective");
}

}  // namespace

int main() {
  test_bytewise();
  test_charwise();
  test_prefix();
  test_bytefold();
  test_string_to_coord_path();
  test_coord_key();
  test_coord_mapn();
  test_mapn_iter();
  test_strategy_edge_cases();
  test_coord_key_roundtrip();

  if (failures != 0) {
    std::fprintf(stderr, "%d check(s) failed\n", failures);
    return 1;
  }
  std::printf("tagma_map: all checks passed\n");
  return 0;
}
