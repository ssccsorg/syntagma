// Verifies CoordSpaceM, the mmap-backed dense space. The checks mirror
// the Rust CoordSpaceM tests in sw/rust/core.

#include <tagma_core/coord_space_m.h>

#include <tagma_core/coord.h>
#include <tagma_core/coord_path.h>

#include <array>
#include <cstdint>
#include <cstdio>
#include <stdexcept>

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

tagma::CoordPath<3> path3(uint16_t a, uint16_t b, uint16_t c) {
  return tagma::CoordPath<3>::from_array({coord(a), coord(b), coord(c)});
}

void test_place_and_at() {
  tagma::CoordSpaceM<3, uint32_t> space;
  check(space.is_empty(), "new space is empty");
  check(space.len() == 0, "new space len is zero");
  const auto p = path3(1, 2, 3);
  check(space.at_path(p) == nullptr, "at_path on empty slot");
  check(!space.place_path(p, 42).has_value(), "place returns no previous");
  check(space.len() == 1, "len after place");
  const uint32_t* value = space.at_path(p);
  check(value != nullptr && *value == 42, "at_path finds value");
}

void test_place_overwrite() {
  tagma::CoordSpaceM<3, uint32_t> space;
  const auto p = path3(5, 5, 5);
  space.place_path(p, 1);
  const auto previous = space.place_path(p, 2);
  check(previous.has_value() && *previous == 1, "overwrite returns previous");
  check(space.len() == 1, "len unchanged on overwrite");
  check(space.at_path(p) != nullptr && *space.at_path(p) == 2,
        "overwrite stores new value");
}

void test_vacate() {
  tagma::CoordSpaceM<3, uint32_t> space;
  const auto p = path3(9, 9, 9);
  space.place_path(p, 7);
  const auto value = space.vacate_path(p);
  check(value.has_value() && *value == 7, "vacate returns value");
  check(space.at_path(p) == nullptr, "vacate empties slot");
  check(space.len() == 0, "len after vacate");
  check(!space.vacate_path(p).has_value(), "vacate on empty slot");
}

void test_clear() {
  tagma::CoordSpaceM<3, uint32_t> space;
  space.place_path(path3(1, 1, 1), 11);
  space.place_path(path3(2, 2, 2), 22);
  check(space.len() == 2, "len before clear");
  space.clear();
  check(space.len() == 0, "len after clear");
  check(space.at_path(path3(1, 1, 1)) == nullptr, "cleared slot one");
  check(space.at_path(path3(2, 2, 2)) == nullptr, "cleared slot two");
}

void test_distinct_paths_distinct_slots() {
  tagma::CoordSpaceM<3, uint16_t> space;
  const auto a = path3(0, 0, 1);
  const auto b = path3(0, 1, 0);
  space.place_path(a, 1);
  space.place_path(b, 2);
  check(space.len() == 2, "distinct paths occupy distinct slots");
  check(space.at_path(a) != nullptr && *space.at_path(a) == 1,
        "value at path a");
  check(space.at_path(b) != nullptr && *space.at_path(b) == 2,
        "value at path b");
}

void test_unsupported_depth() {
  bool threw = false;
  try {
    tagma::CoordSpaceM<6, uint32_t> space;  // 11172^6 overflows size_t
  } catch (const std::invalid_argument&) {
    threw = true;
  }
  check(threw, "unsupported depth throws");
}

}  // namespace

int main() {
  test_place_and_at();
  test_place_overwrite();
  test_vacate();
  test_clear();
  test_distinct_paths_distinct_slots();
  test_unsupported_depth();

  if (failures != 0) {
    std::fprintf(stderr, "%d check(s) failed\n", failures);
    return 1;
  }
  std::printf("tagma_core coord_space_m: all checks passed\n");
  return 0;
}
