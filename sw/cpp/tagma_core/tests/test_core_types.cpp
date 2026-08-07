// Verifies the Tagma C++ core container types: CoordPath, CoordSet, and
// CoordSpace. The checks mirror the Rust tagma-core semantics.

#include <tagma_core/coord.h>
#include <tagma_core/coord_path.h>
#include <tagma_core/coord_set.h>
#include <tagma_core/coord_space.h>

#include <array>
#include <cstdio>
#include <vector>

namespace {

int failures = 0;

void check(bool condition, const char* message) {
  if (!condition) {
    std::fprintf(stderr, "FAIL: %s\n", message);
    failures += 1;
  }
}

tagma::Coord coord(int i, int m, int f) {
  return tagma::Coord::from_axes(i, m, f).value();
}

tagma::Coord coord_index(int index) {
  return tagma::Coord::from_index(static_cast<uint16_t>(index)).value();
}

void test_coord_path() {
  using tagma::CoordPath;
  const std::array<tagma::Coord, 3> coords = {coord(0, 0, 0), coord(1, 2, 3),
                                              coord(18, 20, 27)};
  const CoordPath<3> path = CoordPath<3>::from_array(coords);

  check(path.len() == 3, "path length");
  check(!path.is_empty(), "path not empty");
  check(path.coords() == coords, "path coords array");
  check(*path.get(0) == coords[0], "path get first");
  check(*path.get(2) == coords[2], "path get last");
  check(path.get(3) == nullptr, "path get out of range");
  check(path.get(-1) == nullptr, "path get negative");

  int count = 0;
  for (const tagma::Coord& c : path) {
    (void)c;
    count += 1;
  }
  check(count == 3, "path iteration");

  const CoordPath<0> empty;
  check(empty.len() == 0, "empty path length");
  check(empty.is_empty(), "empty path is empty");
}

void test_coord_set() {
  using tagma::CoordSet;
  CoordSet set;
  check(set.is_empty(), "set initially empty");
  check(set.len() == 0, "set initial length");

  const tagma::Coord a = coord(0, 0, 0);
  const tagma::Coord b = coord(5, 10, 15);
  const tagma::Coord c = coord(18, 20, 27);

  check(set.insert(a), "insert new returns true");
  check(!set.insert(a), "insert duplicate returns false");
  check(set.insert(b), "insert second");
  check(set.len() == 2, "set length after inserts");
  check(set.contains(a) && set.contains(b), "set contains");
  check(!set.contains(c), "set lacks other");

  CoordSet other;
  other.insert(b);
  other.insert(c);
  check(set.is_disjoint(other) == false, "sets not disjoint");

  const CoordSet u = set.set_union(other);
  check(u.len() == 3 && u.contains(a) && u.contains(b) && u.contains(c),
        "union contents");

  const CoordSet inter = set.set_intersection(other);
  check(inter.len() == 1 && inter.contains(b), "intersection contents");

  const CoordSet diff = set.set_difference(other);
  check(diff.len() == 1 && diff.contains(a), "difference contents");

  const CoordSet sym = set.set_symmetric_difference(other);
  check(sym.len() == 2 && sym.contains(a) && sym.contains(c),
        "symmetric difference contents");

  check(set.is_subset(u), "set subset of union");
  check(!u.is_subset(set), "union not subset of set");
  check(u.is_superset(set), "union superset of set");
  check(CoordSet{}.set_union(CoordSet{}).is_empty(), "empty union");

  check(set.remove(a), "remove present returns true");
  check(!set.remove(a), "remove absent returns false");
  check(set.len() == 1, "set length after remove");

  set.clear();
  check(set.is_empty() && set.len() == 0, "set clear");
  check(CoordSet::capacity() == tagma::Coord::kNValid, "set capacity");

  // get, take, retain, iteration, equality.
  set.insert(a);
  set.insert(b);
  check(set.get(a) == std::optional<tagma::Coord>(a), "set get present");
  check(set.get(c) == std::nullopt, "set get absent");
  check(set.take(b) == std::optional<tagma::Coord>(b), "set take present");
  check(set.take(b) == std::nullopt, "set take absent");
  check(set.len() == 1, "set length after take");

  set.insert(b);
  set.retain([](tagma::Coord coord) { return coord.index() % 2 == 0; });
  check(set.contains(a) && !set.contains(b), "set retain parity");

  std::vector<tagma::Coord> seen;
  for (const tagma::Coord coord : set) seen.push_back(coord);
  check(seen.size() == 1 && seen[0] == a, "set iteration");

  CoordSet lhs;
  lhs.insert(a);
  CoordSet rhs;
  rhs.insert(a);
  check(lhs == rhs && !(lhs != rhs), "set equality");
  rhs.insert(b);
  check(lhs != rhs, "set inequality");

  CoordSet full;
  int iterated = 0;
  for (const tagma::Coord coord : full) {
    (void)coord;
    iterated += 1;
  }
  check(iterated == 0, "empty set iteration");
}

void test_coord_space() {
  using tagma::CoordSpace;
  CoordSpace<int> space;
  check(space.is_empty(), "space initially empty");
  check(space.len() == 0, "space initial length");

  const tagma::Coord a = coord(0, 0, 0);
  const tagma::Coord b = coord(5, 10, 15);

  check(space.place(a, 11) == std::nullopt, "first place no previous");
  check(space.place(a, 22) == std::optional<int>(11), "replace returns previous");
  check(space.len() == 1, "space length after place");
  check(space.occupied(a), "space occupied");
  check(!space.occupied(b), "space not occupied at b");
  check(space.at(a) != nullptr && *space.at(a) == 22, "space at");
  check(space.at(b) == nullptr, "space at absent");
  *space.at_mut(a) = 33;
  check(*space.at(a) == 33, "space at_mut");

  check(space.vacate(b) == std::nullopt, "vacate absent");
  check(space.vacate(a) == std::optional<int>(33), "vacate present");
  check(space.is_empty(), "space empty after vacate");

  space.place(a, 1);
  space.place(b, 2);
  space.clear();
  check(space.is_empty() && space.len() == 0, "space clear");
  check(CoordSpace<int>::capacity() == tagma::Coord::kNValid, "space capacity");

  // Path access, entry, retain, iteration.
  const tagma::CoordPath<1> path_a =
      tagma::CoordPath<1>::from_array({a});
  check(space.place_path(path_a, 5) == std::nullopt, "place path");
  check(space.at_path(path_a) != nullptr && *space.at_path(path_a) == 5,
        "at path");
  check(space.vacate_path(path_a) == std::optional<int>(5), "vacate path");

  space.entry(a).or_insert(7);
  space.entry(a).or_insert(9);
  check(*space.at(a) == 7, "entry or_insert keeps existing");
  space.entry(b).or_insert(3);
  check(*space.at(b) == 3, "entry or_insert inserts");
  space.entry(b).or_insert(0) = 3;
  check(*space.at(b) == 3, "entry or_insert mutable reference");

  space.retain([](tagma::Coord, int value) { return value > 5; });
  check(space.occupied(a) && !space.occupied(b), "space retain");

  space.entry(b).or_insert_with([] { return 5; });
  check(*space.at(b) == 5, "space entry or_insert_with");
  check(space.len() == 2, "space length after or_insert_with");
  space.vacate(b);

  int pairs = 0;
  for (const auto& pair : space) {
    check(pair.first == a, "space iteration coord");
    pairs += 1;
  }
  check(pairs == 1, "space iteration count");
}

void test_coord_set_fill_and_remove_all() {
  using tagma::CoordSet;
  // Fill every slot. Mirrors fill_all.
  CoordSet full;
  for (int i = 0; i < tagma::Coord::kNValid; ++i) {
    full.insert(coord_index(i));
  }
  check(full.len() == static_cast<std::size_t>(tagma::Coord::kNValid),
        "set fill all length");
  check(!full.is_empty(), "set fill all not empty");
  bool all_present = true;
  for (int i = 0; i < tagma::Coord::kNValid; ++i) {
    if (!full.contains(coord_index(i))) {
      all_present = false;
      break;
    }
  }
  check(all_present, "set fill all contains every coord");

  // Remove every slot. Mirrors remove_all.
  for (int i = 0; i < tagma::Coord::kNValid; ++i) {
    full.remove(coord_index(i));
  }
  check(full.is_empty() && full.len() == 0, "set remove all");
}

void test_coord_set_from_iterator() {
  using tagma::CoordSet;
  // from-iterator equivalent: collect from a coord vector, deduplicating
  // on insert. Mirrors from_iterator.
  const std::vector<tagma::Coord> input = {coord_index(1), coord_index(2),
                                           coord_index(1)};
  CoordSet set;
  int inserted = 0;
  for (const tagma::Coord c : input) {
    if (set.insert(c)) inserted += 1;
  }
  check(inserted == 2 && set.len() == 2, "set from iterator dedups");
  check(set.contains(coord_index(1)) && set.contains(coord_index(2)),
        "set from iterator contents");
}

void test_coord_set_retain_all_and_empty() {
  using tagma::CoordSet;
  CoordSet set;
  set.insert(coord_index(1));
  set.insert(coord_index(2));
  set.retain([](tagma::Coord) { return true; });
  check(set.len() == 2, "set retain all keeps everything");
  set.retain([](tagma::Coord) { return false; });
  check(set.is_empty(), "set retain none empties");
}

void test_coord_set_copy_eq() {
  using tagma::CoordSet;
  CoordSet original;
  original.insert(coord_index(1));
  const CoordSet copy = original;
  check(copy == original, "set copy equality");
  check(copy.contains(coord_index(1)), "set copy contents");
}

}  // namespace

int main() {
  test_coord_path();
  test_coord_set();
  test_coord_space();
  test_coord_set_fill_and_remove_all();
  test_coord_set_from_iterator();
  test_coord_set_retain_all_and_empty();
  test_coord_set_copy_eq();

  if (failures != 0) {
    std::fprintf(stderr, "%d check(s) failed\n", failures);
    return 1;
  }
  std::printf("tagma_core core types: all checks passed\n");
  return 0;
}
