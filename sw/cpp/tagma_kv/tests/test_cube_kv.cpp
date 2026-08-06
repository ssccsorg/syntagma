// Verifies the CoordCubeKV spatial queries over CoordKVN, CoordKV2, and
// DynCoordKV. The checks mirror the Rust coord_cube_kv tests.

#include <tagma_kv/coord_cube_kv.h>
#include <tagma_kv/coord_kv2.h>
#include <tagma_kv/coord_kv_n.h>
#include <tagma_kv/coord_key.h>
#include <tagma_kv/dyn_coord_kv.h>

#include <tagma_core/coord.h>
#include <tagma_core/coord_path.h>

#include <array>
#include <cstdio>
#include <cstdint>
#include <initializer_list>
#include <string>
#include <utility>
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

template <int N>
tagma::CoordPath<N> path_of(std::initializer_list<uint16_t> indices) {
  std::array<tagma::Coord, N> coords{};
  int i = 0;
  for (const uint16_t index : indices) coords[i++] = coord(index);
  return tagma::CoordPath<N>::from_array(coords);
}

std::vector<uint8_t> bytes(const std::string& s) {
  return std::vector<uint8_t>(s.begin(), s.end());
}

void test_kv2_proximity() {
  using tagma_kv::CoordKV2;
  using tagma_kv::CoordKey;
  CoordKV2 kv;
  const CoordKey<2> center_key(std::array<uint8_t, 2>{{5, 5}});
  const CoordKey<2> nearby_key(std::array<uint8_t, 2>{{5, 6}});
  const CoordKey<2> far_key(std::array<uint8_t, 2>{{5, 20}});
  kv.insert_by_coordkey(center_key, bytes("center"));
  kv.insert_by_coordkey(nearby_key, bytes("nearby"));
  kv.insert_by_coordkey(far_key, bytes("far"));

  const auto results =
      tagma_kv::proximity<2, 2, 1>(kv, center_key.to_coord_path(), 1);
  check(results.size() == 2, "kv2 proximity finds 2");
}

void test_kv2_bounding_box() {
  using tagma_kv::CoordKV2;
  using tagma_kv::CoordKey;
  CoordKV2 kv;
  kv.insert_by_coordkey(CoordKey<2>(std::array<uint8_t, 2>{{5, 5}}),
                        bytes("v1"));
  kv.insert_by_coordkey(CoordKey<2>(std::array<uint8_t, 2>{{5, 6}}),
                        bytes("v2"));
  kv.insert_by_coordkey(CoordKey<2>(std::array<uint8_t, 2>{{10, 10}}),
                        bytes("v3"));

  const std::array<std::pair<uint16_t, uint16_t>, 2> ranges = {
      {{4, 6}, {5, 7}}};
  const auto results = tagma_kv::bounding_box_range(kv, ranges);
  check(results.size() == 2, "kv2 bounding box finds 2");
}

void test_kvn_proximity() {
  using tagma_kv::CoordKVN;
  using tagma_kv::CoordKey;
  CoordKVN<3> kv;
  const auto center_path = path_of<3>({5, 5, 5});
  const auto nearby_path = path_of<3>({5, 5, 6});
  const auto far_path = path_of<3>({5, 5, 20});
  kv.insert_by_coordkey(CoordKey<3>::from_coord_path(center_path),
                        bytes("center"));
  kv.insert_by_coordkey(CoordKey<3>::from_coord_path(nearby_path),
                        bytes("nearby"));
  kv.insert_by_coordkey(CoordKey<3>::from_coord_path(far_path), bytes("far"));

  const auto results = tagma_kv::proximity<3, 3, 1>(kv, center_path, 1);
  check(results.size() == 2, "kvn proximity finds 2");
}

void test_kvn_empty() {
  using tagma_kv::CoordKVN;
  const CoordKVN<2> kv;
  const auto center = path_of<2>({5, 5});
  check(tagma_kv::proximity<2, 2, 1>(kv, center, 1).empty(),
        "kvn empty proximity");
  const std::array<std::pair<uint16_t, uint16_t>, 2> ranges = {
      {{0, 100}, {0, 100}}};
  check(tagma_kv::bounding_box_range(kv, ranges).empty(), "kvn empty box");
}

void test_dynkv_spatial() {
  using tagma_kv::DynCoordKV;
  DynCoordKV kv;
  kv.insert("ab", bytes("v1"));
  kv.insert("ac", bytes("v2"));
  kv.insert("az", bytes("v3"));

  const auto center = path_of<2>({97, 98});  // "ab"
  const auto prox = tagma_kv::proximity<2, 2, 1>(kv, center, 1);
  check(prox.size() == 2, "dynkv proximity finds 2");

  const std::array<std::pair<uint16_t, uint16_t>, 2> ranges = {
      {{97, 99}, {98, 100}}};
  const auto box = tagma_kv::bounding_box_range(kv, ranges);
  check(box.size() == 2, "dynkv bounding box finds 2");

  // "az" sits at (97, 122), so the box must cover index 122 to include
  // all three entries.
  const std::array<std::pair<uint16_t, uint16_t>, 2> wide = {
      {{0, 200}, {0, 200}}};
  check(tagma_kv::bounding_box_range(kv, wide).size() == 3,
        "dynkv wide box finds 3");
}

}  // namespace

int main() {
  test_kv2_proximity();
  test_kv2_bounding_box();
  test_kvn_proximity();
  test_kvn_empty();
  test_dynkv_spatial();

  if (failures != 0) {
    std::fprintf(stderr, "%d check(s) failed\n", failures);
    return 1;
  }
  std::printf("tagma_kv spatial: all checks passed\n");
  return 0;
}
