// Verifies the tagma_kv hashless string-key store: generation strategies,
// CoordKey, and CoordKVN. The checks mirror the Rust tagma-kv coord_gen
// and coord_kv_n tests.

#include <tagma_kv/coord_gen.h>
#include <tagma_kv/coord_key.h>
#include <tagma_kv/coord_kv_n.h>

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
  const auto coords = tagma_kv::ByteWise::generate("hi");
  check(coords.has_value() && coords->size() == 2, "bytewise path length");
  if (coords) {
    check((*coords)[0].index() == 'h' && (*coords)[1].index() == 'i',
          "bytewise indices");
  }
  check(!tagma_kv::ByteWise::generate("").has_value(), "bytewise empty");
  check(tagma_kv::ByteWise::is_injective(), "bytewise injective");
  check(!tagma_kv::ByteWise::fixed_depth().has_value(), "bytewise dynamic");

  // Unicode: UTF-8 bytes map one-to-one.
  const auto korean = tagma_kv::ByteWise::generate("가");
  check(korean.has_value() && korean->size() == 3, "bytewise utf8 bytes");
}

void test_charwise() {
  const auto coords = tagma_kv::CharWise::generate("hi");
  check(coords.has_value() && coords->size() == 4, "charwise two per char");
  if (coords) {
    const uint32_t h = 'h';
    check((*coords)[0].index() == h / 11172 && (*coords)[1].index() == h % 11172,
          "charwise first char split");
  }
  const auto hangul = tagma_kv::CharWise::generate("가");
  check(hangul.has_value() && hangul->size() == 2, "charwise hangul pair");
  check(!tagma_kv::CharWise::generate("").has_value(), "charwise empty");
}

void test_prefix() {
  const auto coords = tagma_kv::Prefix<4>::generate("hi");
  check(coords.has_value() && coords->size() == 4, "prefix length 4");
  if (coords) {
    check((*coords)[0].index() == 'h' && (*coords)[1].index() == 'i' &&
              (*coords)[2].index() == 0 && (*coords)[3].index() == 0,
          "prefix zero pad");
  }
  check(!tagma_kv::Prefix<4>::is_injective(), "prefix lossy");
  check(tagma_kv::Prefix<4>::fixed_depth() == std::optional<std::size_t>(4),
        "prefix fixed depth");
}

void test_bytefold() {
  const auto coords = tagma_kv::ByteFold<2>::generate("ab");
  check(coords.has_value() && coords->size() == 2, "bytefold length");
  if (coords) {
    check((*coords)[0].index() == 'a' && (*coords)[1].index() == 'b',
          "bytefold accumulators");
  }
  const auto again = tagma_kv::ByteFold<2>::generate("ab");
  check(again == coords, "bytefold deterministic");
  check(!tagma_kv::ByteFold<2>::is_injective(), "bytefold lossy");
}

void test_string_to_coord_path() {
  const auto a = tagma_kv::string_to_coord_path("key");
  const auto b = tagma_kv::ByteWise::generate("key");
  check(a.has_value() && a == b, "string_to_coord_path matches bytewise");
  check(!tagma_kv::string_to_coord_path("").has_value(), "empty rejected");
}

void test_coord_key() {
  using tagma_kv::CoordKey;
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

void test_coord_kvn() {
  using tagma_kv::CoordKVN;
  CoordKVN<2> kv;
  check(kv.is_empty() && kv.len() == 0, "kvn initially empty");

  check(kv.insert("hi", bytes("v1")) == std::nullopt, "kvn first insert");
  check(kv.len() == 1, "kvn length");
  check(kv.contains_key("hi"), "kvn contains");
  check(kv.get("hi") == std::optional<std::vector<uint8_t>>(bytes("v1")),
        "kvn get");
  check(kv.get("no") == std::nullopt, "kvn get absent");

  check(kv.insert("hi", bytes("v2")) ==
            std::optional<std::vector<uint8_t>>(bytes("v1")),
        "kvn replace returns previous");
  check(kv.len() == 1, "kvn length after replace");

  check(kv.remove("hi") == std::optional<std::vector<uint8_t>>(bytes("v2")),
        "kvn remove");
  check(kv.is_empty(), "kvn empty after remove");
  check(kv.get("hi") == std::nullopt, "kvn get after remove");

  // Wrong-length keys: get/remove return nullopt, insert throws, mirroring
  // the Rust reference (CoordKV::get/remove guard on length; insert panics
  // via CoordKey::from).
  check(kv.get("x") == std::nullopt, "kvn wrong length get");
  check(kv.remove("x") == std::nullopt, "kvn wrong length remove");
  bool threw = false;
  try {
    kv.insert("x", bytes("v"));
  } catch (const std::invalid_argument&) {
    threw = true;
  }
  check(threw, "kvn wrong length insert throws");

  // CoordKey API.
  const tagma_kv::CoordKey<2> key(std::array<uint8_t, 2>{{'a', 'b'}});
  check(kv.insert_by_coordkey(key, bytes("v3")) == std::nullopt,
        "kvn insert by coordkey");
  check(kv.contains_key_by_coordkey(key), "kvn contains by coordkey");
  check(kv.get_by_coordkey(key) ==
            std::optional<std::vector<uint8_t>>(bytes("v3")),
        "kvn get by coordkey");

  kv.clear();
  check(kv.is_empty() && kv.len() == 0, "kvn clear");
}

void test_kvn_iter() {
  using tagma_kv::CoordKVN;
  const CoordKVN<2> empty;
  check(empty.iter().empty(), "kvn iter empty");

  CoordKVN<2> kv;
  kv.insert("aa", bytes("1"));
  kv.insert("bb", bytes("2"));
  const auto entries = kv.iter();
  check(entries.size() == 2, "kvn iter size");
  check(entries[0].first == std::array<uint8_t, 2>{{'a', 'a'}} &&
            *entries[0].second == bytes("1"),
        "kvn iter first in ascending order");
  check(entries[1].first == std::array<uint8_t, 2>{{'b', 'b'}} &&
            *entries[1].second == bytes("2"),
        "kvn iter second in ascending order");
}

}  // namespace

int main() {
  test_bytewise();
  test_charwise();
  test_prefix();
  test_bytefold();
  test_string_to_coord_path();
  test_coord_key();
  test_coord_kvn();
  test_kvn_iter();

  if (failures != 0) {
    std::fprintf(stderr, "%d check(s) failed\n", failures);
    return 1;
  }
  std::printf("tagma_kv: all checks passed\n");
  return 0;
}
