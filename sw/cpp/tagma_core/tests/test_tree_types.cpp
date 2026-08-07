// Verifies the CoordSpaceN tree and the CoordSetN sparse set: lazy
// allocation, path-based access, N=1 single-coordinate access, entry API,
// path collection, and set operations. The checks mirror the Rust
// coord_space_n.rs and coord_set_n.rs reference semantics.

#include <tagma_core/coord.h>
#include <tagma_core/coord_path.h>
#include <tagma_core/coord_set_n.h>
#include <tagma_core/coord_space_n.h>

#include <array>
#include <cassert>
#include <cstdint>
#include <cstdio>
#include <initializer_list>
#include <optional>
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

tagma::Coord coord(uint16_t index) {
  return tagma::Coord::from_index(index).value();
}

template <int N>
tagma::CoordPath<N> path_of(std::initializer_list<uint16_t> indices) {
  assert(indices.size() <= static_cast<std::size_t>(N) &&
         "path_of: too many indices for depth N");
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

void test_coord_set_n_edge_cases() {
  using tagma::CoordSetN;

  // from-iterator equivalent: collect from a path vector, deduplicating
  // on insert. Mirrors from_iterator_collects_all and
  // from_iterator_deduplicates.
  const std::vector<tagma::CoordPath<2>> input = {
      path_of<2>({1, 2}), path_of<2>({3, 4}), path_of<2>({1, 2})};
  CoordSetN<2> collected;
  int inserted = 0;
  for (const auto& path : input) {
    if (collected.insert(path)) inserted += 1;
  }
  check(inserted == 2 && collected.len() == 2, "setn from iterator dedups");
  check(collected.contains(path_of<2>({1, 2})) &&
            collected.contains(path_of<2>({3, 4})),
        "setn from iterator contents");

  // Empty-identity operations.
  CoordSetN<2> a;
  a.insert(path_of<2>({1, 2}));
  const CoordSetN<2> empty;
  check(a.set_union(empty).len() == 1 &&
            a.set_union(empty).contains(path_of<2>({1, 2})),
        "setn union with empty returns self");
  check(a.set_intersection(empty).is_empty(),
        "setn intersection with empty is empty");
  check(a.set_difference(empty).len() == 1,
        "setn difference with empty returns self");
  check(a.set_symmetric_difference(empty).len() == 1,
        "setn symdiff with empty returns self");
  check(empty.is_subset(a) && !a.is_subset(empty), "setn subset with empty");
  check(empty.is_disjoint(a), "setn disjoint with empty");

  // clear then reinsert.
  a.clear();
  check(a.is_empty(), "setn clear");
  a.insert(path_of<2>({3, 4}));
  check(a.contains(path_of<2>({3, 4})) && a.len() == 1,
        "setn reinsert after clear");

  // Iteration order determinism: reverse insert order yields ascending
  // paths. Mirrors iter_tree_order_deterministic.
  CoordSetN<2> ordered;
  for (int i = 19; i >= 0; --i) {
    ordered.insert(path_of<2>({static_cast<uint16_t>(i), 0}));
  }
  const auto paths = ordered.paths();
  check(paths.size() == 20, "setn iter count");
  bool ascending = true;
  for (std::size_t i = 1; i < paths.size(); ++i) {
    if (paths[i - 1].coords()[0].index() > paths[i].coords()[0].index()) {
      ascending = false;
    }
  }
  check(ascending, "setn iter ascending");

  // Iteration count equals len. Mirrors iter_tree_yields_same_as_len.
  CoordSetN<2> many;
  for (int i = 0; i < 30; ++i) {
    many.insert(path_of<2>({static_cast<uint16_t>(i),
                            static_cast<uint16_t>(i + 50)}));
  }
  check(many.paths().size() == 30 && many.len() == 30,
        "setn iter same as len");
}

void test_coord_space_n6() {
  using tagma::CoordSpaceN6;
  CoordSpaceN6<std::string> space;
  const auto path = path_of<6>({0, 1, 2, 3, 4, 5});
  check(space.at_path(path) == nullptr, "n6 missing path");
  check(space.place_path(path, std::string("hello")) == std::nullopt,
        "n6 place");
  check(space.at_path(path) != nullptr && *space.at_path(path) == "hello",
        "n6 at");
  check(space.len() == 1, "n6 len");
  check(space.vacate_path(path) == std::optional<std::string>("hello"),
        "n6 vacate");
  check(space.is_empty(), "n6 empty after vacate");
}

void test_coord_space_n12_and_max_depth() {
  using tagma::CoordSpaceN12;
  CoordSpaceN12<uint32_t> twelve;
  const auto p12 = path_of<12>({0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11});
  check(twelve.place_path(p12, 42) == std::nullopt, "n12 place");
  check(twelve.at_path(p12) != nullptr && *twelve.at_path(p12) == 42,
        "n12 at");

  using tagma::CoordSpaceN19;
  CoordSpaceN19<uint32_t> max_depth;
  const auto p19 = path_of<19>({0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13,
                                14, 15, 16, 17, 18});
  check(max_depth.place_path(p19, 42) == std::nullopt,
        "n19 max depth place");
  check(max_depth.at_path(p19) != nullptr && *max_depth.at_path(p19) == 42,
        "n19 max depth at");
  check(max_depth.len() == 1, "n19 max depth len");
}

void test_type_aliases() {
  tagma::CoordSpaceN1<uint32_t> m1;
  tagma::CoordSpaceN2<uint32_t> m2;
  tagma::CoordSpaceN3<uint32_t> m3;
  tagma::CoordSpaceN6<uint32_t> m6;
  tagma::CoordSpaceN12<uint32_t> m12;
  tagma::CoordSpaceN19<uint32_t> m19;
  check(m1.is_empty() && m2.is_empty() && m3.is_empty() && m6.is_empty() &&
            m12.is_empty() && m19.is_empty(),
        "type aliases exist and start empty");
}

void test_entries_order_deterministic() {
  // Reverse insert order; entries() must yield ascending depth-first
  // order. Mirrors iter_tree_order_is_deterministic.
  tagma::CoordSpaceN2<uint32_t> space;
  for (int i = 99; i >= 0; --i) {
    space.place_path(path_of<2>({static_cast<uint16_t>(i), 0}),
                     static_cast<uint32_t>(i));
  }
  const auto entries = space.entries();
  check(entries.size() == 100, "entries count");
  bool ascending = true;
  for (std::size_t i = 1; i < entries.size(); ++i) {
    if (entries[i - 1].first.coords()[0].index() >
        entries[i].first.coords()[0].index()) {
      ascending = false;
    }
  }
  check(ascending, "entries ascending order");
}

}  // namespace

int main() {
  test_coord_space_n1();
  test_coord_space_n2();
  test_coord_set_n3();
  test_coord_set_n_edge_cases();
  test_coord_space_n6();
  test_coord_space_n12_and_max_depth();
  test_type_aliases();
  test_entries_order_deterministic();

  if (failures != 0) {
    std::fprintf(stderr, "%d check(s) failed\n", failures);
    return 1;
  }
  std::printf("tagma_core tree types: all checks passed\n");
  return 0;
}
