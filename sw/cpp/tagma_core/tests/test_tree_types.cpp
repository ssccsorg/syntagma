// Verifies the CoordSpaceN tree and the CoordSetN sparse set: lazy
// allocation, path-based access, N=1 single-coordinate access, entry API,
// path collection, and set operations. The checks mirror the Rust
// coord_space_n.rs and coord_set_n.rs reference semantics.

#include <tagma_core/coord.h>
#include <tagma_core/coord_path.h>
#include <tagma_core/coord_set_n.h>
#include <tagma_core/coord_space_n.h>

#include <array>
#include <cstdio>
#include <initializer_list>
#include <optional>

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
  for (const uint16_t index : indices) {
    coords[i] = coord(index);
    i += 1;
  }
  return tagma::CoordPath<N>::from_array(coords);
}

void test_coord_space_n1() {
  using tagma::CoordSpaceN1;
  CoordSpaceN1<int> space;

  check(space.is_empty() && space.len() == 0, "n1 initially empty");
  check(space.capacity() == std::optional<std::size_t>(11172), "n1 capacity");

  const tagma::Coord a = coord(0);
  const tagma::Coord b = coord(3235);

  check(space.place(a, 11) == std::nullopt, "n1 first place");
  check(space.place(a, 22) == std::optional<int>(11), "n1 replace");
  check(space.len() == 1, "n1 length");
  check(space.occupied(a) && !space.occupied(b), "n1 occupied");
  check(space.at(a) != nullptr && *space.at(a) == 22, "n1 at");
  *space.at_mut(a) = 33;
  check(*space.at(a) == 33, "n1 at_mut");
  check(space.vacate(b) == std::nullopt, "n1 vacate absent");
  check(space.vacate(a) == std::optional<int>(33), "n1 vacate present");
  check(space.is_empty(), "n1 empty after vacate");

  space.entry(a).or_insert(7);
  space.entry(a).or_insert(9);
  check(*space.at(a) == 7, "n1 entry or_insert keeps existing");
  space.entry(b).or_insert_with([] { return 5; });
  check(*space.at(b) == 5, "n1 entry or_insert_with");

  check(space.paths().size() == 2, "n1 paths size");
  space.clear();
  check(space.is_empty() && space.len() == 0, "n1 clear");
}

void test_coord_space_n2() {
  using tagma::CoordSpaceN2;
  CoordSpaceN2<int> space;

  check(space.is_empty() && space.len() == 0, "n2 initially empty");
  check(space.capacity() == std::nullopt, "n2 capacity nullopt");

  const tagma::CoordPath<2> pa = path_of<2>({0, 1});
  const tagma::CoordPath<2> pb = path_of<2>({3235, 11171});
  const tagma::CoordPath<2> pc = path_of<2>({0, 11171});

  check(space.at_path(pa) == nullptr, "n2 at absent");
  check(space.place_path(pa, 10) == std::nullopt, "n2 first place");
  check(space.place_path(pa, 20) == std::optional<int>(10), "n2 replace");
  check(space.len() == 1, "n2 length");
  check(space.at_path(pa) != nullptr && *space.at_path(pa) == 20, "n2 at path");
  *space.at_path_mut(pa) = 30;
  check(*space.at_path(pa) == 30, "n2 at_path_mut");

  space.place_path(pb, 40);
  check(space.len() == 2, "n2 length after second");
  check(space.at_path(pc) == nullptr, "n2 sibling absent");

  check(space.vacate_path(pb) == std::optional<int>(40), "n2 vacate path");
  check(space.vacate_path(pb) == std::nullopt, "n2 vacate absent");
  check(space.len() == 1, "n2 length after vacate");

  check(space.paths().size() == 1 && space.paths()[0] == pa, "n2 paths");
  space.clear();
  check(space.is_empty() && space.len() == 0, "n2 clear");
}

void test_coord_set_n3() {
  using tagma::CoordSetN;
  CoordSetN<3> set;

  const tagma::CoordPath<3> a = path_of<3>({0, 0, 0});
  const tagma::CoordPath<3> b = path_of<3>({1, 2, 3});
  const tagma::CoordPath<3> c = path_of<3>({18, 20, 27});

  check(set.is_empty(), "setn3 initially empty");
  check(set.insert(a), "setn3 insert new");
  check(!set.insert(a), "setn3 insert duplicate");
  check(set.insert(b), "setn3 insert second");
  check(set.len() == 2, "setn3 length");
  check(set.contains(a) && set.contains(b) && !set.contains(c), "setn3 contains");
  check(set.remove(b), "setn3 remove present");
  check(!set.remove(b), "setn3 remove absent");
  check(set.len() == 1, "setn3 length after remove");
  set.insert(b);
  set.insert(c);

  CoordSetN<3> other;
  other.insert(b);
  other.insert(c);

  const CoordSetN<3> u = set.set_union(other);
  check(u.len() == 3, "setn3 union size");

  const CoordSetN<3> inter = set.set_intersection(other);
  check(inter.len() == 2 && inter.contains(b) && inter.contains(c),
        "setn3 intersection");

  const CoordSetN<3> diff = set.set_difference(other);
  check(diff.len() == 1 && diff.contains(a), "setn3 difference");

  const CoordSetN<3> sym = set.set_symmetric_difference(other);
  check(sym.len() == 1 && sym.contains(a), "setn3 symmetric difference");

  check(set.is_subset(u) && u.is_superset(set), "setn3 subset/superset");
  check(!set.is_disjoint(other), "setn3 not disjoint");
  CoordSetN<3> only_a;
  only_a.insert(a);
  check(only_a.is_disjoint(other), "setn3 disjoint");
  check(set != only_a, "setn3 inequality");

  check(set.paths().size() == 3, "setn3 paths size");
  set.clear();
  check(set.is_empty() && set.len() == 0, "setn3 clear");
}

}  // namespace

int main() {
  test_coord_space_n1();
  test_coord_space_n2();
  test_coord_set_n3();

  if (failures != 0) {
    std::fprintf(stderr, "%d check(s) failed\n", failures);
    return 1;
  }
  std::printf("tagma_core tree types: all checks passed\n");
  return 0;
}
