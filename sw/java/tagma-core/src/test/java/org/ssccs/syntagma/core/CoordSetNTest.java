package org.ssccs.syntagma.core;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.List;
import org.junit.jupiter.api.Test;

/**
 * Translation of the CoordSetN section of the C++ suite
 * {@code sw/cpp/tagma_core/tests/test_tree_types.cpp}, extended with the
 * behaviors that only the Rust suite {@code sw/rust/core/tests/coord_set_n.rs}
 * covers.
 */
class CoordSetNTest {

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

    @Test
    void coordSetN3() {
        CoordSetN set = new CoordSetN(3);

        CoordPath a = path(0, 0, 0);
        CoordPath b = path(1, 2, 3);
        CoordPath c = path(18, 20, 27);

        assertTrue(set.isEmpty(), "setn3 initially empty");
        assertTrue(set.insert(a), "setn3 insert new");
        assertFalse(set.insert(a), "setn3 insert duplicate");
        assertTrue(set.insert(b), "setn3 insert second");
        assertEquals(2, set.size(), "setn3 length");
        assertTrue(set.contains(a) && set.contains(b) && !set.contains(c), "setn3 contains");
        assertTrue(set.remove(b), "setn3 remove present");
        assertFalse(set.remove(b), "setn3 remove absent");
        assertEquals(1, set.size(), "setn3 length after remove");
        set.insert(b);
        set.insert(c);

        CoordSetN other = new CoordSetN(3);
        other.insert(b);
        other.insert(c);

        CoordSetN union = set.union(other);
        assertEquals(3, union.size(), "setn3 union size");

        CoordSetN intersection = set.intersection(other);
        assertEquals(2, intersection.size(), "setn3 intersection size");
        assertTrue(intersection.contains(b) && intersection.contains(c), "setn3 intersection content");

        CoordSetN difference = set.difference(other);
        assertEquals(1, difference.size(), "setn3 difference size");
        assertTrue(difference.contains(a), "setn3 difference content");

        CoordSetN symmetric = set.symmetricDifference(other);
        assertEquals(1, symmetric.size(), "setn3 symmetric difference size");
        assertTrue(symmetric.contains(a), "setn3 symmetric difference content");

        assertTrue(set.isSubset(union) && union.isSuperset(set), "setn3 subset and superset");
        assertFalse(set.isDisjoint(other), "setn3 not disjoint");

        CoordSetN onlyA = new CoordSetN(3);
        onlyA.insert(a);
        assertTrue(onlyA.isDisjoint(other), "setn3 disjoint");
        assertNotEquals(set, onlyA, "setn3 inequality");

        assertEquals(3, set.paths().size(), "setn3 paths size");
        set.clear();
        assertTrue(set.isEmpty() && set.size() == 0, "setn3 clear");
    }

    @Test
    void insertionFromAPathListDeduplicates() {
        List<CoordPath> input = List.of(path(1, 2), path(3, 4), path(1, 2));
        CoordSetN collected = new CoordSetN(2);
        int inserted = 0;
        for (CoordPath candidate : input) {
            if (collected.insert(candidate)) {
                inserted += 1;
            }
        }
        assertEquals(2, inserted, "setn from iterator insertion count");
        assertEquals(2, collected.size(), "setn from iterator size");
        assertTrue(collected.contains(path(1, 2)) && collected.contains(path(3, 4)),
                "setn from iterator contents");
    }

    @Test
    void emptyIdentityOperations() {
        CoordSetN a = new CoordSetN(2);
        a.insert(path(1, 2));
        CoordSetN empty = new CoordSetN(2);

        CoordSetN union = a.union(empty);
        assertEquals(1, union.size(), "setn union with empty returns self");
        assertTrue(union.contains(path(1, 2)), "setn union with empty content");
        assertTrue(a.intersection(empty).isEmpty(), "setn intersection with empty is empty");
        assertEquals(1, a.difference(empty).size(), "setn difference with empty returns self");
        assertEquals(1, a.symmetricDifference(empty).size(), "setn symmetric difference with empty returns self");
        assertTrue(empty.isSubset(a) && !a.isSubset(empty), "setn subset with empty");
        assertTrue(empty.isDisjoint(a) && a.isDisjoint(empty), "setn disjoint with empty");

        CoordSetN emptyCopy = empty.copy();
        assertTrue(emptyCopy.isEmpty(), "setn copy of empty");
        a.clear();
        assertTrue(a.isEmpty(), "setn clear");
        a.insert(path(3, 4));
        assertTrue(a.contains(path(3, 4)) && a.size() == 1, "setn reinsert after clear");
    }

    @Test
    void unionDeduplicates() {
        CoordSetN a = new CoordSetN(2);
        a.insert(path(1, 2));
        CoordSetN b = new CoordSetN(2);
        b.insert(path(1, 2));
        b.insert(path(3, 4));
        assertEquals(2, a.union(b).size(), "setn union deduplicates");
    }

    @Test
    void intersectionIteratesSmallerSet() {
        CoordSetN large = new CoordSetN(2);
        for (int i = 0; i < 100; i++) {
            large.insert(path(i, 0));
        }
        CoordSetN small = new CoordSetN(2);
        small.insert(path(1, 0));
        CoordSetN intersection = small.intersection(large);
        assertEquals(1, intersection.size(), "setn intersection size");
        assertTrue(intersection.contains(path(1, 0)), "setn intersection content");
        assertTrue(small.intersection(new CoordSetN(2)).isEmpty(), "setn disjoint intersection is empty");
    }

    @Test
    void symmetricDifferenceExclusiveOnly() {
        CoordSetN a = new CoordSetN(2);
        a.insert(path(1, 2));
        a.insert(path(3, 4));
        CoordSetN b = new CoordSetN(2);
        b.insert(path(3, 4));
        b.insert(path(5, 6));
        CoordSetN symmetric = a.symmetricDifference(b);
        assertEquals(2, symmetric.size(), "setn symmetric difference size");
        assertTrue(symmetric.contains(path(1, 2)), "setn symmetric difference exclusive left");
        assertFalse(symmetric.contains(path(3, 4)), "setn symmetric difference drops the common path");
        assertTrue(symmetric.contains(path(5, 6)), "setn symmetric difference exclusive right");
    }

    @Test
    void subsetAndDisjointRejections() {
        CoordSetN a = new CoordSetN(2);
        a.insert(path(1, 2));
        a.insert(path(5, 6));
        CoordSetN b = new CoordSetN(2);
        b.insert(path(1, 2));
        assertFalse(a.isSubset(b), "setn subset false");
        assertTrue(b.isSubset(a), "setn subset true");
        assertFalse(a.isDisjoint(b), "setn disjoint false");
        assertTrue(b.isDisjoint(new CoordSetN(2)), "setn disjoint with empty is true");
    }

    @Test
    void equalityIgnoresInsertionOrder() {
        CoordSetN a = new CoordSetN(2);
        a.insert(path(1, 2));
        a.insert(path(3, 4));
        CoordSetN b = new CoordSetN(2);
        b.insert(path(3, 4));
        b.insert(path(1, 2));
        assertEquals(a, b, "setn equality ignores insertion order");
        assertEquals(a.hashCode(), b.hashCode(), "setn equal sets share a hash");
        CoordSetN c = new CoordSetN(2);
        c.insert(path(9, 9));
        assertNotEquals(a, c, "setn inequality with different content");
        assertNotEquals(a, "not a set", "setn inequality with a foreign type");
    }

    @Test
    void iterationIsDeterministicAndComplete() {
        CoordSetN ordered = new CoordSetN(2);
        for (int i = 19; i >= 0; i--) {
            ordered.insert(path(i, 0));
        }
        List<CoordPath> paths = ordered.paths();
        assertEquals(20, paths.size(), "setn iter count");
        for (int i = 1; i < paths.size(); i++) {
            int previous = paths.get(i - 1).get(0).orElseThrow().index();
            int current = paths.get(i).get(0).orElseThrow().index();
            assertTrue(previous <= current, "setn iter ascending");
        }

        CoordSetN many = new CoordSetN(2);
        for (int i = 0; i < 30; i++) {
            many.insert(path(i, i + 50));
        }
        assertEquals(30, many.paths().size(), "setn iter yields every path");
        assertEquals(many.size(), many.paths().size(), "setn iter count equals len");

        assertEquals(0, new CoordSetN(2).paths().size(), "setn iter of an empty set yields nothing");
    }

    @Test
    void copyIsIndependent() {
        CoordSetN original = new CoordSetN(2);
        original.insert(path(1, 2));
        original.insert(path(5, 6));
        CoordSetN copy = original.copy();
        copy.insert(path(3, 4));
        original.remove(path(1, 2));
        assertFalse(original.contains(path(1, 2)), "setn copy source mutation");
        assertTrue(copy.contains(path(1, 2)), "setn copy keeps source content");
        assertTrue(copy.contains(path(3, 4)), "setn copy accepts new content");
        assertEquals(1, original.size(), "setn copy source size");
        assertEquals(3, copy.size(), "setn copy size");
    }

    @Test
    void argumentsValidated() {
        CoordSetN set = new CoordSetN(2);
        assertThrows(IllegalArgumentException.class, () -> set.insert(path(1)), "wrong path depth rejected");
        assertThrows(IllegalArgumentException.class, () -> new CoordSetN(0), "depth 0 rejected");
        CoordSetN otherDepth = new CoordSetN(3);
        assertThrows(IllegalArgumentException.class, () -> set.union(otherDepth), "depth mismatch rejected");
        assertThrows(IllegalArgumentException.class, () -> set.intersection(otherDepth), "depth mismatch rejected");
        assertThrows(IllegalArgumentException.class, () -> set.difference(otherDepth), "depth mismatch rejected");
        assertThrows(IllegalArgumentException.class, () -> set.symmetricDifference(otherDepth),
                "depth mismatch rejected");
        assertThrows(IllegalArgumentException.class, () -> set.isSubset(otherDepth), "depth mismatch rejected");
        assertThrows(IllegalArgumentException.class, () -> set.isSuperset(otherDepth), "depth mismatch rejected");
        assertThrows(IllegalArgumentException.class, () -> set.isDisjoint(otherDepth), "depth mismatch rejected");
        assertThrows(NullPointerException.class, () -> set.insert(null), "null path rejected");
    }
}
