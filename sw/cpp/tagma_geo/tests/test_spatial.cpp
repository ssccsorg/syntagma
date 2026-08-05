// Verifies the tagma_geo spatial operations: BoundingBoxIter, HammingFilter,
// and the SpatialOps / DistanceMetrics functions over CoordCube. The checks
// mirror the Rust tagma-geo spatial.rs tests.

#include <tagma_geo/spatial.h>

#include <tagma_core/coord.h>
#include <tagma_core/coord_cube.h>
#include <tagma_core/coord_path.h>

#include <array>
#include <cmath>
#include <cstdio>
#include <cstdint>
#include <initializer_list>
#include <stdexcept>

namespace {

int failures = 0;

void check(bool condition, const char* message) {
  if (!condition) {
    std::fprintf(stderr, "FAIL: %s\n", message);
    failures += 1;
  }
}

void check_close(double a, double b, const char* message) {
  if (std::fabs(a - b) > 1e-9) {
    std::fprintf(stderr, "FAIL: %s (%.9f vs %.9f)\n", message, a, b);
    failures += 1;
  }
}

tagma::Coord coord(uint16_t index) {
  return tagma::Coord::from_index(index).value();
}

template <int N, int D, int R>
tagma::CoordCube<N, D, R> cube_of(std::initializer_list<uint16_t> indices) {
  std::array<tagma::Coord, N> coords{};
  int i = 0;
  for (const uint16_t index : indices) coords[i++] = coord(index);
  return tagma::CoordCube<N, D, R>::from_path(
      tagma::CoordPath<N>::from_array(coords));
}

int count_paths(const tagma_geo::BoundingBoxIter<2>& box) {
  int count = 0;
  for (const tagma::CoordPath<2>& path : box) {
    (void)path;
    count += 1;
  }
  return count;
}

void test_bb_iter_single_character() {
  const tagma_geo::BoundingBoxIter<1> box(
      std::array<std::pair<uint16_t, uint16_t>, 1>{{{3, 3}}});
  check(box.count_paths() == 1, "bb single count");
  check(!box.is_empty(), "bb single not empty");
  int count = 0;
  for (const tagma::CoordPath<1>& path : box) {
    check(path.coords()[0].index() == 3, "bb single index");
    count += 1;
  }
  check(count == 1, "bb single iteration");
}

void test_bb_iter_two_characters() {
  const tagma_geo::BoundingBoxIter<2> box(
      std::array<std::pair<uint16_t, uint16_t>, 2>{{{0, 1}, {0, 1}}});
  check(box.count_paths() == 4, "bb two count");
  int count = 0;
  for (const tagma::CoordPath<2>& path : box) {
    (void)path;
    count += 1;
  }
  check(count == 4, "bb two iteration");
}

void test_bb_iter_max_range() {
  const tagma_geo::BoundingBoxIter<1> box(
      std::array<std::pair<uint16_t, uint16_t>, 1>{{{0, 11171}}});
  check(box.count_paths() == 11172, "bb max count");
  int count = 0;
  for (const tagma::CoordPath<1>& path : box) {
    (void)path;
    count += 1;
  }
  check(count == 11172, "bb max iteration");
}

void test_bb_iter_empty_n0() {
  const tagma_geo::BoundingBoxIter<0> box(std::array<std::pair<uint16_t, uint16_t>, 0>{});
  check(box.is_empty(), "bb n0 empty");
  int count = 0;
  for (const tagma::CoordPath<0>& path : box) {
    (void)path;
    count += 1;
  }
  check(count == 0, "bb n0 iteration");
}

void test_bb_iter_exhaustion() {
  tagma_geo::BoundingBoxIter<1> box(
      std::array<std::pair<uint16_t, uint16_t>, 1>{{{0, 1}}});
  check(!box.is_empty(), "bb exhaustion not empty at start");
  ++box;
  ++box;
  check(box.is_empty(), "bb exhaustion empty after two steps");
}

void test_bb_iter_invalid_ranges() {
  bool threw = false;
  try {
    tagma_geo::BoundingBoxIter<1> inverted(
        std::array<std::pair<uint16_t, uint16_t>, 1>{{{5, 3}}});
    (void)inverted;
  } catch (const std::invalid_argument&) {
    threw = true;
  }
  check(threw, "bb inverted range throws");

  threw = false;
  try {
    tagma_geo::BoundingBoxIter<1> oob(
        std::array<std::pair<uint16_t, uint16_t>, 1>{{{0, 11172}}});
    (void)oob;
  } catch (const std::invalid_argument&) {
    threw = true;
  }
  check(threw, "bb out of bounds range throws");
}

void test_bb_count_paths() {
  const tagma_geo::BoundingBoxIter<2> box(
      std::array<std::pair<uint16_t, uint16_t>, 2>{{{0, 2}, {0, 3}}});
  check(box.count_paths() == 12, "bb count 3x4");
}

void test_cube_bounding_box_basic() {
  const auto cube = cube_of<2, 2, 1>({0, 0});
  const auto box = tagma_geo::bounding_box(
      cube, std::array<std::pair<uint16_t, uint16_t>, 2>{{{0, 1}, {2, 3}}});
  check(box.count_paths() == 4, "cube bb count");
  check(count_paths(box) == 4, "cube bb iteration");
}

void test_cube_proximity() {
  const auto cube = cube_of<2, 2, 1>({5, 10});

  const auto radius0 = tagma_geo::proximity(cube, 0);
  check(radius0.count_paths() == 1, "proximity r0 count");

  const auto radius1 = tagma_geo::proximity(cube, 1);
  check(radius1.count_paths() == 9, "proximity r1 count");

  // Clamp to bounds: cube at index 0.
  const auto edge = cube_of<2, 2, 1>({0, 11171});
  const auto clamped = tagma_geo::proximity(edge, 3);
  check(clamped.count_paths() == 4 * 4, "proximity clamp count");
}

void test_hamming() {
  const auto a = cube_of<2, 2, 1>({0, 0});
  const auto b = cube_of<2, 2, 1>({0, 5});
  check(tagma_geo::hamming_distance(a, b) == 1, "hamming differ one");
  check(tagma_geo::hamming_distance(a, a) == 0, "hamming identical");

  const auto all_diff = cube_of<2, 2, 1>({3, 4});
  check(tagma_geo::hamming_distance(a, all_diff) == 2, "hamming all differ");
}

void test_hamming_axes() {
  // dim0 = coords[0..1], dim1 = coords[2..3].
  const auto a = cube_of<4, 2, 2>({0, 0, 0, 0});
  const auto b = cube_of<4, 2, 2>({0, 1, 2, 3});
  const std::array<std::size_t, 2> axes = tagma_geo::hamming_distance_axes(a, b);
  check(axes[0] == 1 && axes[1] == 2, "hamming axes values");
}

void test_euclidean() {
  const auto a = cube_of<2, 2, 1>({0, 0});
  check_close(tagma_geo::euclidean_distance_approx(a, a), 0.0,
              "euclidean identical");

  const auto b = cube_of<2, 2, 1>({11171, 0});
  check_close(tagma_geo::euclidean_distance_approx(a, b), 1.0,
              "euclidean max in one dim");
}

void test_manhattan() {
  const auto a = cube_of<2, 2, 1>({0, 0});
  check(tagma_geo::manhattan_distance(a, a) == 0, "manhattan identical");

  const auto b = cube_of<2, 2, 1>({5, 0});
  check(tagma_geo::manhattan_distance(a, b) == 5, "manhattan different");

  const auto c = cube_of<4, 2, 2>({0, 0, 0, 0});
  const auto d = cube_of<4, 2, 2>({1, 0, 0, 0});
  check(tagma_geo::manhattan_distance(c, d) == 1, "manhattan multi character");
}

void test_proximity_hamming() {
  const auto cube = cube_of<2, 2, 1>({5, 10});

  int count0 = 0;
  for (const tagma::CoordPath<2>& path :
       tagma_geo::proximity_hamming(cube, 0)) {
    check(path == cube.as_path(), "proximity hamming r0 only center");
    count0 += 1;
  }
  check(count0 == 1, "proximity hamming r0 count");

  int count1 = 0;
  for (const tagma::CoordPath<2>& path :
       tagma_geo::proximity_hamming(cube, 1)) {
    (void)path;
    count1 += 1;
  }
  // Center + 4 axis neighbors at Hamming distance 1 within radius 1 box.
  check(count1 == 5, "proximity hamming r1 count");
}

void test_hamming_filter_direct() {
  const tagma_geo::BoundingBoxIter<2> box(
      std::array<std::pair<uint16_t, uint16_t>, 2>{{{0, 2}, {0, 2}}});
  const auto center = cube_of<2, 2, 1>({1, 1}).as_path();
  tagma_geo::HammingFilter<2> filter(box, center, 1);
  int count = 0;
  for (const tagma::CoordPath<2>& path : filter) {
    (void)path;
    count += 1;
  }
  // In a 3x3 box around center (1,1): center + 4 axis neighbors.
  check(count == 5, "hamming filter direct count");
}

}  // namespace

int main() {
  test_bb_iter_single_character();
  test_bb_iter_two_characters();
  test_bb_iter_max_range();
  test_bb_iter_empty_n0();
  test_bb_iter_exhaustion();
  test_bb_iter_invalid_ranges();
  test_bb_count_paths();
  test_cube_bounding_box_basic();
  test_cube_proximity();
  test_hamming();
  test_hamming_axes();
  test_euclidean();
  test_manhattan();
  test_proximity_hamming();
  test_hamming_filter_direct();

  if (failures != 0) {
    std::fprintf(stderr, "%d check(s) failed\n", failures);
    return 1;
  }
  std::printf("tagma_geo spatial: all checks passed\n");
  return 0;
}
