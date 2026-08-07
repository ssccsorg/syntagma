// Verifies the Tagma coordinate engine C++ core: composition,
// decomposition, validity, axes, code point conversion, and Hamming
// distance.

#include <tagma_core/coord.h>

#include <cstdint>
#include <cstdio>
#include <functional>
#include <sstream>
#include <string>
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

  // constexpr evaluation of the closed-form arithmetic.
  static_assert(Coord::from_axes(0, 0, 0).has_value());
  static_assert(Coord::from_axes(5, 10, 15)->index() == 3235);
  static_assert(Coord::from_axes(5, 10, 15)->code_point() == 0xB8A3);
  static_assert(Coord::from_index(0)->to_char() == U'가');
  static_assert(Coord::from_axes(1, 0, 0) < Coord::from_axes(2, 0, 0));
  static_assert(Coord::from_axes(18, 20, 27)->hamming_distance(
                    Coord::from_axes(0, 0, 0).value()) ==
                std::make_tuple(18, 20, 27));

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
  check(Coord::kTotal == 65536, "total states constant");

  // UTF-8 display of the compositional character.
  check(Coord::from_index(0)->to_hangul_string() == "가",
        "hangul string base");
  check(Coord::from_index(11171)->to_hangul_string() == "힣",
        "hangul string end");

  // Byte serialization of the raw index.
  const std::array<uint8_t, 2> le_last = {0xA3, 0x2B};  // 11171 LE
  const std::array<uint8_t, 2> be_last = {0x2B, 0xA3};  // 11171 BE
  check(Coord::from_index(11171)->to_le_bytes() == le_last, "le bytes");
  check(Coord::from_index(11171)->to_be_bytes() == be_last, "be bytes");
  check(Coord::from_le_bytes(le_last).has_value() &&
            Coord::from_le_bytes(le_last)->index() == 11171,
        "from le bytes");
  check(Coord::from_be_bytes(be_last).has_value() &&
            Coord::from_be_bytes(be_last)->index() == 11171,
        "from be bytes");
  const std::array<uint8_t, 2> le_invalid = {0xA4, 0x2B};  // 11172 LE
  const std::array<uint8_t, 2> be_invalid = {0x2B, 0xA4};  // 11172 BE
  check(!Coord::from_le_bytes(le_invalid).has_value(),
        "invalid le bytes rejected");
  check(!Coord::from_be_bytes(be_invalid).has_value(),
        "invalid be bytes rejected");

  // Ordering by index.
  const Coord low = Coord::from_index(0).value();
  const Coord high = Coord::from_index(11171).value();
  check(low < high && low <= high && high > low && high >= low,
        "ordering operators");
  check(low != high, "inequality");
  check(low == low, "equality");

  // Hash by index.
  check(std::hash<Coord>{}(low) == 0, "hash of index zero");
  check(std::hash<Coord>{}(high) == 11171, "hash of last index");

  // Stream display.
  std::ostringstream out;
  out << Coord::from_index(0).value();
  check(out.str() == "가", "stream display");

  // Exhaustive lattice scan: every index 0..11171 is valid and
  // roundtrips through axes, code points, and characters; the first
  // invalid index is rejected. Mirrors the Rust
  // all_11172_coords_are_valid.
  bool scan_valid = true;
  bool scan_roundtrip = true;
  for (int index = 0; index < Coord::kNValid; ++index) {
    const auto lattice_coord = Coord::from_index(static_cast<uint16_t>(index));
    if (!lattice_coord) {
      scan_valid = false;
      break;
    }
    const auto [initial, medial, final] = lattice_coord->axes();
    if (Coord::from_axes(initial, medial, final) != *lattice_coord ||
        Coord::from_code_point(lattice_coord->code_point()) != *lattice_coord ||
        Coord::from_char(lattice_coord->to_char()) != *lattice_coord) {
      scan_roundtrip = false;
      break;
    }
  }
  check(scan_valid, "all 11172 indices valid");
  check(scan_roundtrip, "axes/code point/char roundtrip over all indices");
  check(!Coord::from_index(static_cast<uint16_t>(Coord::kNValid)).has_value(),
        "first invalid index rejected");

  if (failures != 0) {
    std::fprintf(stderr, "%d check(s) failed\n", failures);
    return 1;
  }
  std::printf("tagma_core coord: all checks passed\n");
  return 0;
}
