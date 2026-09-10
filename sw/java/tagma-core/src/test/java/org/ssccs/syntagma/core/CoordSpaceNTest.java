package org.ssccs.syntagma.core;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.OptionalInt;
import org.junit.jupiter.api.Test;

/**
 * Translation of the CoordSpaceN section of the C++ suite
 * {@code sw/cpp/tagma_core/tests/test_tree_types.cpp}, extended with the
 * behaviors that only the Rust suite
 * {@code sw/rust/core/tests/coord_space_n.rs} covers.
 */
class CoordSpaceNTest {

    private static Coord coord(int index) {
        return Coord.fromIndex(index).orElseThrow();
    }

    private static CoordPath path(int... indices) {
        Coord[] coords = new Coord[indices.length];
        for (int i = 0; i < indices.length; i++) {
            coords[i] = coord(indices[i]);
        }
        return CoordPath.fromArray(coords);
    }

    private static CoordPath path19() {
        return path(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18);
    }

    // ------------------------------------------------------------------
    // Depth 1
    // ------------------------------------------------------------------

    @Test
    void coordSpaceN1() {
        CoordSpaceN<Integer> space = new CoordSpaceN<>(1);

        assertTrue(space.isEmpty() && space.size() == 0, "n1 initially empty");
        assertEquals(OptionalInt.of(11172), space.capacity(), "n1 capacity");
        assertEquals(0, space.paths().size(), "n1 empty iteration");
        assertEquals(0, space.entries().size(), "n1 empty entry iteration");

        Coord a = coord(0);
        Coord b = coord(3235);

        assertEquals(Optional.empty(), space.place(a, 11), "n1 first place");
        assertEquals(Optional.of(11), space.place(a, 22), "n1 replace");
        assertEquals(1, space.size(), "n1 length");
        assertTrue(space.occupied(a) && !space.occupied(b), "n1 occupied");
        assertEquals(Optional.of(22), space.at(a), "n1 at");
        space.atMut(a).orElseThrow().set(33);
        assertEquals(Optional.of(33), space.at(a), "n1 at_mut");
        assertEquals(Optional.empty(), space.vacate(b), "n1 vacate absent");
        assertEquals(Optional.of(33), space.vacate(a), "n1 vacate present");
        assertTrue(space.isEmpty(), "n1 empty after vacate");

        space.entry(a).orInsert(7);
        space.entry(a).orInsert(9);
        assertEquals(Optional.of(7), space.at(a), "n1 entry or_insert keeps existing");
        space.entry(b).orInsertWith(() -> 5);
        assertEquals(Optional.of(5), space.at(b), "n1 entry or_insert_with");

        assertEquals(2, space.paths().size(), "n1 paths size");
        space.clear();
        assertTrue(space.isEmpty() && space.size() == 0, "n1 clear");
    }

    @Test
    void depthOnePathAccess() {
        CoordSpaceN<Integer> space = new CoordSpaceN<>(1);
        Coord c = coord(42);

        space.place(c, 100);
        assertEquals(Optional.of(100), space.atPath(path(42)), "flat get path");

        space.clear();
        assertEquals(Optional.empty(), space.placePath(path(42), 100), "flat insert path");
        assertEquals(Optional.of(100), space.at(c), "flat insert path visible at coord");

        assertEquals(Optional.of(100), space.vacatePath(path(42)), "flat remove path");
        assertTrue(space.isEmpty(), "flat remove path empties the space");
    }

    // ------------------------------------------------------------------
    // Depth 2
    // ------------------------------------------------------------------

    @Test
    void coordSpaceN2() {
        CoordSpaceN<Integer> space = new CoordSpaceN<>(2);

        assertTrue(space.isEmpty() && space.size() == 0, "n2 initially empty");
        assertTrue(space.capacity().isEmpty(), "n2 capacity empty");

        CoordPath pa = path(0, 1);
        CoordPath pb = path(3235, 11171);
        CoordPath pc = path(0, 11171);

        assertEquals(Optional.empty(), space.atPath(pa), "n2 at absent");
        assertEquals(Optional.empty(), space.placePath(pa, 10), "n2 first place");
        assertEquals(Optional.of(10), space.placePath(pa, 20), "n2 replace");
        assertEquals(1, space.size(), "n2 length");
        assertEquals(Optional.of(20), space.atPath(pa), "n2 at path");
        space.atPathMut(pa).orElseThrow().set(30);
        assertEquals(Optional.of(30), space.atPath(pa), "n2 at_path_mut");

        space.placePath(pb, 40);
        assertEquals(2, space.size(), "n2 length after second");
        assertEquals(Optional.empty(), space.atPath(pc), "n2 sibling absent");

        assertEquals(Optional.of(40), space.vacatePath(pb), "n2 vacate path");
        assertEquals(Optional.empty(), space.vacatePath(pb), "n2 vacate absent");
        assertEquals(1, space.size(), "n2 length after vacate");

        List<CoordPath> paths = space.paths();
        assertEquals(1, paths.size(), "n2 paths size");
        assertEquals(pa, paths.get(0), "n2 paths content");
        space.clear();
        assertTrue(space.isEmpty() && space.size() == 0, "n2 clear");
    }

    @Test
    void siblingPathsSurviveVacate() {
        CoordSpaceN<Integer> space = new CoordSpaceN<>(2);
        CoordPath pathA = path(0, 0);
        CoordPath pathB = path(0, 1);
        space.placePath(pathA, 10);
        space.placePath(pathB, 20);
        space.vacatePath(pathA);
        assertEquals(1, space.size(), "sibling length after vacate");
        assertEquals(Optional.empty(), space.atPath(pathA), "vacated path absent");
        assertEquals(Optional.of(20), space.atPath(pathB), "sibling value kept");
    }

    @Test
    void clearThenReuse() {
        CoordSpaceN<Integer> space = new CoordSpaceN<>(2);
        space.placePath(path(0, 0), 1);
        space.placePath(path(1, 1), 2);
        assertEquals(2, space.size(), "reuse length before clear");
        space.clear();
        assertTrue(space.isEmpty() && space.size() == 0, "reuse empty after clear");
        space.placePath(path(2, 2), 3);
        assertEquals(Optional.of(3), space.atPath(path(2, 2)), "reuse after clear");
    }

    // ------------------------------------------------------------------
    // Deeper spaces
    // ------------------------------------------------------------------

    @Test
    void coordSpaceN6() {
        CoordSpaceN<String> space = new CoordSpaceN<>(6);
        CoordPath path = path(0, 1, 2, 3, 4, 5);
        assertEquals(Optional.empty(), space.atPath(path), "n6 missing path");
        assertEquals(Optional.empty(), space.placePath(path, "hello"), "n6 place");
        assertEquals(Optional.of("hello"), space.atPath(path), "n6 at");
        assertEquals(1, space.size(), "n6 len");
        assertEquals(Optional.of("hello"), space.vacatePath(path), "n6 vacate");
        assertTrue(space.isEmpty(), "n6 empty after vacate");
    }

    @Test
    void coordSpaceN12AndMaxDepth() {
        CoordSpaceN<Integer> twelve = new CoordSpaceN<>(12);
        CoordPath p12 = path(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11);
        assertEquals(Optional.empty(), twelve.placePath(p12, 42), "n12 place");
        assertEquals(Optional.of(42), twelve.atPath(p12), "n12 at");

        CoordSpaceN<Integer> maxDepth = new CoordSpaceN<>(19);
        CoordPath p19 = path19();
        assertEquals(Optional.empty(), maxDepth.placePath(p19, 42), "n19 max depth place");
        assertEquals(Optional.of(42), maxDepth.atPath(p19), "n19 max depth at");
        assertEquals(1, maxDepth.size(), "n19 max depth len");
    }

    @Test
    void depthAliasesExistAndStartEmpty() {
        for (int depth : new int[] {1, 2, 3, 6, 12, 19}) {
            CoordSpaceN<Integer> space = new CoordSpaceN<>(depth);
            assertTrue(space.isEmpty(), "depth " + depth + " alias starts empty");
            assertEquals(depth, space.depth(), "depth " + depth + " alias keeps its depth");
        }
    }

    // ------------------------------------------------------------------
    // Iteration
    // ------------------------------------------------------------------

    @Test
    void entriesOrderDeterministic() {
        CoordSpaceN<Integer> space = new CoordSpaceN<>(2);
        for (int i = 99; i >= 0; i--) {
            space.placePath(path(i, 0), i);
        }
        List<Map.Entry<CoordPath, Integer>> entries = space.entries();
        assertEquals(100, entries.size(), "entries count");
        for (int i = 1; i < entries.size(); i++) {
            int previous = entries.get(i - 1).getKey().get(0).orElseThrow().index();
            int current = entries.get(i).getKey().get(0).orElseThrow().index();
            assertTrue(previous <= current, "entries ascending order");
        }
    }

    @Test
    void entriesMatchAtPath() {
        CoordSpaceN<Integer> space = new CoordSpaceN<>(2);
        for (int i = 0; i < 50; i++) {
            space.placePath(path(i, i + 100), i);
        }
        List<Map.Entry<CoordPath, Integer>> entries = space.entries();
        assertEquals(50, entries.size(), "iter_tree yields all entries");
        for (Map.Entry<CoordPath, Integer> entry : entries) {
            assertTrue(entry.getKey().length() == 2, "iter_tree yields full-depth paths");
            assertEquals(Optional.of(entry.getValue()), space.atPath(entry.getKey()),
                    "iter_tree path matches at_path");
        }
        assertEquals(50, space.paths().size(), "paths and entries agree");
    }

    @Test
    void entriesPrefixSubset() {
        CoordSpaceN<Integer> space = new CoordSpaceN<>(2);
        space.placePath(path(42, 1), 10);
        space.placePath(path(42, 2), 20);
        space.placePath(path(99, 0), 30);

        List<Map.Entry<CoordPath, Integer>> under = space.entriesPrefix(List.of(coord(42))).orElseThrow();
        assertEquals(2, under.size(), "prefix subset size");
        assertEquals(path(42, 1), under.get(0).getKey(), "prefix subset first path");
        assertEquals(path(42, 2), under.get(1).getKey(), "prefix subset second path");
        assertEquals(List.of(10, 20), List.of(under.get(0).getValue(), under.get(1).getValue()));

        assertTrue(space.entriesPrefix(List.of(coord(11111))).isEmpty(),
                "missing prefix yields no subtree");
        assertTrue(space.entriesPrefix(List.of(coord(0), coord(0))).isEmpty(),
                "prefix of full depth yields no subtree");
    }

    @Test
    void entriesPrefixOfExistingEmptySubtree() {
        CoordSpaceN<Integer> space = new CoordSpaceN<>(2);
        space.placePath(path(1, 1), 1);
        space.vacatePath(path(1, 1));
        List<Map.Entry<CoordPath, Integer>> under = space.entriesPrefix(List.of(coord(1))).orElseThrow();
        assertTrue(under.isEmpty(), "allocated but empty subtree yields no entries");
    }

    // ------------------------------------------------------------------
    // Rust-only behavior
    // ------------------------------------------------------------------

    @Test
    void copyIsIndependent() {
        CoordSpaceN<Integer> a = new CoordSpaceN<>(2);
        a.placePath(path(0, 0), 1);
        CoordSpaceN<Integer> b = a.copy();
        b.placePath(path(1, 1), 2);
        assertEquals(1, a.size(), "clone source length");
        assertEquals(2, b.size(), "clone target length");
        assertEquals(Optional.of(1), a.atPath(path(0, 0)), "clone source content");
        assertEquals(Optional.of(1), b.atPath(path(0, 0)), "clone target shares source content");
        assertEquals(Optional.of(2), b.atPath(path(1, 1)), "clone target addition");
    }

    @Test
    void structuralEqualityMirrorsRustPartialEq() {
        CoordSpaceN<Integer> a = new CoordSpaceN<>(2);
        CoordSpaceN<Integer> b = new CoordSpaceN<>(2);
        a.placePath(path(0, 0), 42);
        b.placePath(path(0, 0), 42);
        assertEquals(a, b, "same tree structure compares equal");
        assertEquals(a.hashCode(), b.hashCode(), "equal trees share a hash");

        b.placePath(path(1, 1), 99);
        assertNotEquals(a, b, "extra node compares unequal");

        CoordSpaceN<Integer> vacated = new CoordSpaceN<>(2);
        vacated.placePath(path(0, 0), 7);
        vacated.vacatePath(path(0, 0));
        assertNotEquals(vacated, new CoordSpaceN<Integer>(2),
                "nodes are never pruned, mirroring the structural Rust PartialEq");
        vacated.clear();
        assertEquals(vacated, new CoordSpaceN<Integer>(2),
                "clear discards the tree, mirroring the C++ port");

        assertNotEquals(new CoordSpaceN<Integer>(1), new CoordSpaceN<Integer>(2),
                "different depths compare unequal");
        assertNotEquals(a, null, "null compares unequal");
        assertNotEquals(a, "not a space", "foreign types compare unequal");
    }

    @Test
    void debugFormat() {
        CoordSpaceN<Integer> space = new CoordSpaceN<>(2);
        space.placePath(path(0, 0), 1);
        String text = space.toString();
        assertTrue(text.contains("CoordSpace"), "debug contains the type name");
        assertTrue(text.contains("N: 2"), "debug contains the depth");
        assertTrue(text.contains("len: 1"), "debug contains the length");
    }

    @Test
    void entryAndModify() {
        CoordSpaceN<Integer> space = new CoordSpaceN<>(1);
        Coord c = coord(0);
        space.entry(c).andModify(value -> value + 1).orInsert(1);
        assertEquals(Optional.of(1), space.at(c), "and_modify leaves a vacant slot to or_insert");
        space.entry(c).andModify(value -> value + 1).orInsert(1);
        assertEquals(Optional.of(2), space.at(c), "and_modify updates an occupied slot");
    }

    @Test
    void valueRefRejectsVacatedEntry() {
        CoordSpaceN<Integer> space = new CoordSpaceN<>(1);
        Coord c = coord(7);
        CoordSpaceN.ValueRef<Integer> ref = space.entry(c).orInsert(7);
        assertEquals(Optional.of(7), space.at(c), "entry ref get");
        ref.update(value -> value + 1);
        assertEquals(Optional.of(8), space.at(c), "entry ref update");
        space.vacate(c);
        assertThrows(java.util.NoSuchElementException.class, ref::get, "ref get after vacate");
        assertThrows(java.util.NoSuchElementException.class, () -> ref.set(9), "ref set after vacate");
    }

    // ------------------------------------------------------------------
    // Argument validation
    // ------------------------------------------------------------------

    @Test
    void pathLengthMustMatchDepth() {
        CoordSpaceN<Integer> space = new CoordSpaceN<>(2);
        assertThrows(IllegalArgumentException.class, () -> space.atPath(path(1)), "short path rejected");
        assertThrows(IllegalArgumentException.class, () -> space.placePath(path(1, 2, 3), 1), "long path rejected");
        assertThrows(IllegalArgumentException.class, () -> space.vacatePath(path(1)), "short vacate rejected");
        assertThrows(IllegalArgumentException.class, () -> new CoordSpaceN<Integer>(0), "depth 0 rejected");
        assertThrows(IllegalArgumentException.class, () -> new CoordSpaceN<Integer>(-1), "negative depth rejected");
    }

    @Test
    void singleCoordinateAccessRequiresDepthOne() {
        CoordSpaceN<Integer> space = new CoordSpaceN<>(2);
        Coord c = coord(0);
        assertThrows(UnsupportedOperationException.class, () -> space.at(c), "at needs depth 1");
        assertThrows(UnsupportedOperationException.class, () -> space.atMut(c), "atMut needs depth 1");
        assertThrows(UnsupportedOperationException.class, () -> space.occupied(c), "occupied needs depth 1");
        assertThrows(UnsupportedOperationException.class, () -> space.place(c, 1), "place needs depth 1");
        assertThrows(UnsupportedOperationException.class, () -> space.vacate(c), "vacate needs depth 1");
        assertThrows(UnsupportedOperationException.class, () -> space.entry(c), "entry needs depth 1");
    }

    @Test
    void nullArgumentsRejected() {
        CoordSpaceN<Integer> space = new CoordSpaceN<>(1);
        Coord c = coord(0);
        assertThrows(NullPointerException.class, () -> space.place(c, null), "null value");
        assertThrows(NullPointerException.class, () -> space.atPath(null), "null path");
        assertThrows(NullPointerException.class, () -> space.entriesPrefix(null), "null prefix");

        CoordSpaceN<Integer> threeLevels = new CoordSpaceN<>(3);
        assertThrows(NullPointerException.class,
                () -> threeLevels.entriesPrefix(java.util.Arrays.asList((Coord) null)),
                "null prefix coordinate");
    }

    @Test
    void entriesPrefixRequiresAnAllocatedSubtree() {
        CoordSpaceN<Integer> space = new CoordSpaceN<>(2);
        assertTrue(space.entriesPrefix(List.of(coord(0))).isEmpty(),
                "a prefix that was never placed has no subtree");
        assertTrue(space.entriesPrefix(List.of()).isPresent(),
                "the empty prefix addresses the whole tree");
        space.placePath(path(0, 1), 1);
        assertTrue(space.entriesPrefix(List.of(coord(0))).isPresent(),
                "an allocated subtree is reported as present");
    }
}
