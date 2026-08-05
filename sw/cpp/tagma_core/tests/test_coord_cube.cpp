// Verifies the CoordCube interpretation layer: construction, axis
// decomposition, coordinate access, equality, and display. The checks
// mirror the Rust CoordCube<N, D, R> reference semantics.

#include <tagma_core/coord.h>
#include <tagma_core/coord_cube.h>
#include <tagma_core/coord_path.h>

#include <array>
#include <cstdio>
#include <sstream>
#include <string>

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

// The Rust reference example: N = 6, D = 3, R = 2, coords 0..5.
tagma::CoordCube<6, 3, 2> example_cube() {
  const std::array<tagma::Coord, 6> coords = {
      coord(0), coord(1), coord(2), coord(3), coord(4), coord(5)};
  return tagma::CoordCube<6, 3, 2>::from_path(
      tagma::CoordPath<6>::from_array(coords));
}

}  // namespace

int main() {
  using tagma::CoordCube;
  using tagma::CoordPath;

  const CoordCube<6, 3, 2> cube = example_cube();

  // Dimensions and resolution.
  check(cube.ndim() == 3, "cube ndim");
  check(cube.resolution() == 2, "cube resolution");
  check(cube.total_characters() == 6, "cube total characters");

  // Axis decomposition, matching the Rust doc example.
  const CoordPath<2> axis0 = cube.axis(0);
  check(axis0.coords()[0].index() == 0, "axis 0 char 0");
  check(axis0.coords()[1].index() == 1, "axis 0 char 1");
  check(cube.axis(1).coords()[0].index() == 2, "axis 1 char 0");
  check(cube.axis(1).coords()[1].index() == 3, "axis 1 char 1");
  check(cube.axis(2).coords()[0].index() == 4, "axis 2 char 0");
  check(cube.axis(2).coords()[1].index() == 5, "axis 2 char 1");

  // Coordinate access.
  check(cube.coord_at(0, 0).index() == 0, "coord at origin");
  check(cube.coord_at(2, 1).index() == 5, "coord at last");
  check(cube.coords().size() == 6, "cube coords size");

  // Path conversion round trip.
  check(cube.as_path().coords() == cube.coords(), "as_path round trip");
  check(cube.into_path() == cube.as_path(), "into_path round trip");

  // Equality delegates to the underlying path.
  const CoordCube<6, 3, 2> same = example_cube();
  check(cube == same, "cube equality");
  const std::array<tagma::Coord, 6> other_coords = {
      coord(5), coord(4), coord(3), coord(2), coord(1), coord(0)};
  const CoordCube<6, 3, 2> other = CoordCube<6, 3, 2>::from_path(
      CoordPath<6>::from_array(other_coords));
  check(cube != other, "cube inequality");

  // Display mirrors the Rust format.
  std::ostringstream cube_out;
  cube_out << cube;
  check(cube_out.str() == "CoordCube<6, 3, 2>[(가, 각) | (갂, 갃) | (간, 갅)]",
        "cube display");

  std::ostringstream path_out;
  path_out << CoordPath<3>::from_array(
      std::array<tagma::Coord, 3>{coord(0), coord(1), coord(2)});
  check(path_out.str() == "CoordPath<3>(가, 각, 갂)", "path display");

  if (failures != 0) {
    std::fprintf(stderr, "%d check(s) failed\n", failures);
    return 1;
  }
  std::printf("tagma_core coord cube: all checks passed\n");
  return 0;
}
