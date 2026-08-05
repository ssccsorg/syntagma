// Verifies the Tagma coordinate engine C++ core: composition,
// decomposition, validity, axes, code point conversion, and Hamming
// distance.

#include <tagma_core/coord.h>

#include <cstdint>
#include <cstdio>
#include <tuple>

namespace {

int failures = 0;

void check(bool condition, const char* message) {
  if (!condition) {
    std::fprintf(stderr, "FAIL: %s\n", message);
    failures += 1;
  }
}

void check_axes(int i, int m, int f) {
  const auto coord = tagma::Coord::from_axes(i, m, f);
  check(coord.has_value(), "from_axes valid");
  if (!coord) return;
  check(coord->axes() == std::make_tuple(i, m, f), "axes round trip");
  const int expected_index = i * 588 + m * 28 + f;
  check(coord->index() == expected_index, "index composition");
  check(coord->code_point() == tagma::Coord::kBase + expected_index,
        "code point composition");
  check(coord->valid(), "constructed coord valid");
}

}  // namespace

int main() {
  using tagma::Coord;

  // Lattice edges and a midpoint.
  check_axes(0, 0, 0);
  check_axes(18, 20, 27);
  check_axes(5, 10, 15);
  check_axes(0, 20, 27);
  check_axes(18, 0, 27);
  check_axes(18, 20, 0);

  // Invalid axes are rejected.
  check(!Coord::from_axes(-1, 0, 0).has_value(), "negative initial rejected");
  check(!Coord::from_axes(19, 0, 0).has_value(), "initial overflow rejected");
  check(!Coord::from_axes(0, 21, 0).has_value(), "medial overflow rejected");
  check(!Coord::from_axes(0, 0, 28).has_value(), "final overflow rejected");

  // Index bounds.
  check(Coord::from_index(0).has_value(), "index 0 valid");
  check(Coord::from_index(11171).has_value(), "index 11171 valid");
  check(!Coord::from_index(11172).has_value(), "index 11172 rejected");
  check(Coord::from_index(0)->to_char() == U'가', "block base char");
  check(Coord::from_index(11171)->to_char() == U'힣', "block end char");
  check(Coord::from_index(0)->code_point() == 0xAC00, "block base code point");
  check(Coord::from_index(11171)->code_point() == 0xD7A3,
        "block end code point");

  // Code point bounds, including the filler positions U+D7A4..U+D7AF.
  check(Coord::from_code_point(0xAC00).has_value(), "code point base valid");
  check(Coord::from_code_point(0xD7A3).has_value(), "code point end valid");
  check(!Coord::from_code_point(0xABFF).has_value(), "below base rejected");
  check(!Coord::from_code_point(0xD7A4).has_value(), "filler start rejected");
  check(!Coord::from_code_point(0xD7AF).has_value(), "filler end rejected");
  check(!Coord::from_code_point(0xD7B0).has_value(), "above block rejected");
  check(!Coord::from_char(U'ힰ').has_value(), "out of block char rejected");

  // Hamming distance is per-axis absolute difference.
  const auto a = Coord::from_axes(0, 0, 0).value();
  const auto b = Coord::from_axes(1, 0, 0).value();
  const auto c = Coord::from_axes(0, 1, 0).value();
  const auto d = Coord::from_axes(0, 0, 1).value();
  const auto e = Coord::from_axes(3, 5, 7).value();
  check(a.hamming_distance(a) == std::make_tuple(0, 0, 0), "self hamming zero");
  check(a.hamming_distance(b) == std::make_tuple(1, 0, 0),
        "initial axis difference");
  check(a.hamming_distance(c) == std::make_tuple(0, 1, 0),
        "medial axis difference");
  check(a.hamming_distance(d) == std::make_tuple(0, 0, 1),
        "final axis difference");
  check(a.hamming_distance(e) == std::make_tuple(3, 5, 7),
        "three-axis difference");
  check(b.hamming_distance(c) == std::make_tuple(1, 1, 0),
        "two-axis difference");

  // Invalidity margin: 65,536 - 11,172.
  check(Coord::kInvalidMargin == 65536 - Coord::kNValid,
        "invalidity margin constant");

  if (failures != 0) {
    std::fprintf(stderr, "%d check(s) failed\n", failures);
    return 1;
  }
  std::printf("tagma_core coord: all checks passed\n");
  return 0;
}
