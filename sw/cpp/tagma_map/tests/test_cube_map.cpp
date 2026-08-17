// Verifies the CoordCubeMap spatial queries over CoordMapN, CoordMap2, and
// DynCoordMap. The checks mirror the Rust coord_cube_map tests.

#include <tagma_map/coord_cube_map.h>
#include <tagma_map/coord_map2.h>
#include <tagma_map/coord_map_n.h>
#include <tagma_map/coord_key.h>
#include <tagma_map/dyn_coord_map.h>

#include <tagma_core/coord.h>
#include <tagma_core/coord_path.h>

#include <array>
#include <cassert>
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
  assert(indices.size() <= static_cast<std::size_t>(N) &&
         "path_of: too many indices for depth N");
  std::array<tagma::Coord, N> coords{};
  int i = 0;
  for (const uint16_t index : indices) coords[i++] = coord(index);
  return tagma::CoordPath<N>::from_array(coords);
}

std::vector<uint8_t> bytes(const std::string& s) {
  return std::vector<uint8_t>(s.begin(), s.end());
}

void test_map2_proximity() {
  using tagma_map::CoordMap2;
  using tagma_map::CoordKey;
  CoordMap2 map;
  const CoordKey<2> center_key(std::array<uint8_t, 2>{{5, 5}});
  const CoordKey<2> nearby_key(std::array<uint8_t, 2>{{5, 6}});
  const CoordKey<2> far_key(std::array<uint8_t, 2>{{5, 20}});
  map.insert_by_coordkey(center_key, bytes("center"));
  map.insert_by_coordkey(nearby_key, bytes("nearby"));
  map.insert_by_coordkey(far_key, bytes("far"));

  const auto results =
      tagma_map::proximity<2, 2, 1>(map, center_key.to_coord_path(), 1);
  check(results.size() == 2, "map2 proximity finds 2");
}

void test_map2_bounding_box() {
  using tagma_map::CoordMap2;
  using tagma_map::CoordKey;
  CoordMap2 map;
  map.insert_by_coordkey(CoordKey<2>(std::array<uint8_t, 2>{{5, 5}}),
                        bytes("v1"));
  map.insert_by_coordkey(CoordKey<2>(std::array<uint8_t, 2>{{5, 6}}),
                        bytes("v2"));
  map.insert_by_coordkey(CoordKey<2>(std::array<uint8_t, 2>{{10, 10}}),
                        bytes("v3"));

  const std::array<std::pair<uint16_t, uint16_t>, 2> ranges = {
      {{4, 6}, {5, 7}}};
  const auto results = tagma_map::bounding_box_range(map, ranges);
  check(results.size() == 2, "map2 bounding box finds 2");
}

void test_mapn_proximity() {
  using tagma_map::CoordMapN;
  using tagma_map::CoordKey;
  CoordMapN<3> map;
  const auto center_path = path_of<3>({5, 5, 5});
  const auto nearby_path = path_of<3>({5, 5, 6});
  const auto far_path = path_of<3>({5, 5, 20});
  map.insert_by_coordkey(CoordKey<3>::from_coord_path(center_path),
                        bytes("center"));
  map.insert_by_coordkey(CoordKey<3>::from_coord_path(nearby_path),
                        bytes("nearby"));
  map.insert_by_coordkey(CoordKey<3>::from_coord_path(far_path), bytes("far"));

  const auto results = tagma_map::proximity<3, 3, 1>(map, center_path, 1);
  check(results.size() == 2, "mapn proximity finds 2");
}

void test_mapn_empty() {
  using tagma_map::CoordMapN;
  const CoordMapN<2> map;
  const auto center = path_of<2>({5, 5});
  check(tagma_map::proximity<2, 2, 1>(map, center, 1).empty(),
        "mapn empty proximity");
  const std::array<std::pair<uint16_t, uint16_t>, 2> ranges = {
      {{0, 100}, {0, 100}}};
  check(tagma_map::bounding_box_range(map, ranges).empty(), "mapn empty box");
}

void test_dynmap_spatial() {
  using tagma_map::DynCoordMap;
  DynCoordMap map;
  map.insert("ab", bytes("v1"));
  map.insert("ac", bytes("v2"));
  map.insert("az", bytes("v3"));

  const auto center = path_of<2>({97, 98});  // "ab"
  const auto prox = tagma_map::proximity<2, 2, 1>(map, center, 1);
  check(prox.size() == 2, "dynmap proximity finds 2");

  const std::array<std::pair<uint16_t, uint16_t>, 2> ranges = {
      {{97, 99}, {98, 100}}};
  const auto box = tagma_map::bounding_box_range(map, ranges);
  check(box.size() == 2, "dynmap bounding box finds 2");

  // "az" sits at (97, 122), so the box must cover index 122 to include
  // all three entries.
  const std::array<std::pair<uint16_t, uint16_t>, 2> wide = {
      {{0, 200}, {0, 200}}};
  check(tagma_map::bounding_box_range(map, wide).size() == 3,
        "dynmap wide box finds 3");
}

}  // namespace

int main() {
  test_map2_proximity();
  test_map2_bounding_box();
  test_mapn_proximity();
  test_mapn_empty();
  test_dynmap_spatial();

  if (failures != 0) {
    std::fprintf(stderr, "%d check(s) failed\n", failures);
    return 1;
  }
  std::printf("tagma_map spatial: all checks passed\n");
  return 0;
}
