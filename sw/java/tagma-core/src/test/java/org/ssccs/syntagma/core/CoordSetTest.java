package org.ssccs.syntagma.core;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.Optional;
import org.junit.jupiter.api.Test;

/**
 * Translation of the CoordSet section of the C++ test suite
 * {@code sw/cpp/tagma_core/tests/test_core_types.cpp}.
 */
class CoordSetTest {

    private static Coord coord(int initial, int medial, int final_) {
        return Coord.fromAxes(initial, medial, final_).orElseThrow();
    }

    private static Coord coordIndex(int index) {
        return Coord.fromIndex(index).orElseThrow();
    }

    @Test
    void setBasics() {
        CoordSet set = new CoordSet();
        assertTrue(set.isEmpty(), "set initially empty");
        assertEquals(0, set.size(), "set initial size");

        Coord a = coord(0, 0, 0);
        Coord b = coord(5, 10, 15);
        Coord c = coord(18, 20, 27);

        assertTrue(set.insert(a), "insert new returns true");
        assertFalse(set.insert(a), "insert duplicate returns false");
        assertTrue(set.insert(b), "insert second");
        assertEquals(2, set.size(), "set size after inserts");
        assertTrue(set.contains(a) && set.contains(b), "set contains");
        assertFalse(set.contains(c), "set lacks other");
    }

    @Test
    void setOperations() {
        Coord a = coord(0, 0, 0);
        Coord b = coord(5, 10, 15);
        Coord c = coord(18, 20, 27);

        CoordSet set = new CoordSet();
        set.insert(a);
        set.insert(b);
        CoordSet other = new CoordSet();
        other.insert(b);
        other.insert(c);
        assertFalse(set.isDisjoint(other), "sets not disjoint");

        CoordSet union = set.union(other);
        assertEquals(3, union.size());
        assertTrue(union.contains(a) && union.contains(b) && union.contains(c), "union contents");

        CoordSet intersection = set.intersection(other);
        assertEquals(1, intersection.size());
        assertTrue(intersection.contains(b), "intersection contents");

        CoordSet difference = set.difference(other);
        assertEquals(1, difference.size());
        assertTrue(difference.contains(a), "difference contents");

        CoordSet symmetric = set.symmetricDifference(other);
        assertEquals(2, symmetric.size());
        assertTrue(symmetric.contains(a) && symmetric.contains(c), "symmetric difference contents");

        assertTrue(set.isSubset(union), "set subset of union");
        assertFalse(union.isSubset(set), "union not subset of set");
        assertTrue(union.isSuperset(set), "union superset of set");
        assertTrue(new CoordSet().union(new CoordSet()).isEmpty(), "empty union");
    }

    @Test
    void removeAndClear() {
        Coord a = coord(0, 0, 0);
        CoordSet set = new CoordSet();
        set.insert(a);
        assertTrue(set.remove(a), "remove present returns true");
        assertFalse(set.remove(a), "remove absent returns false");
        assertTrue(set.isEmpty() && set.size() == 0, "set empty after remove");

        Coord b = coord(5, 10, 15);
        set.insert(a);
        set.insert(b);
        set.clear();
        assertTrue(set.isEmpty() && set.size() == 0, "set clear");
    }

    @Test
    void capacityConstant() {
        assertEquals(Coord.N_VALID, CoordSet.capacity());
    }

    @Test
    void getTakeRetainIterationEquality() {
        Coord a = coord(0, 0, 0);
        Coord b = coord(5, 10, 15);
        Coord c = coord(18, 20, 27);

        CoordSet set = new CoordSet();
        set.insert(a);
        set.insert(b);
        assertEquals(Optional.of(a), set.get(a), "set get present");
        assertEquals(Optional.empty(), set.get(c), "set get absent");
        assertEquals(Optional.of(b), set.take(b), "set take present");
        assertEquals(Optional.empty(), set.take(b), "set take absent");
        assertEquals(1, set.size(), "set size after take");

        set.insert(b);
        set.retain(coord -> coord.index() % 2 == 0);
        assertTrue(set.contains(a) && !set.contains(b), "set retain parity");

        List<Coord> seen = new ArrayList<>();
        for (Coord coord : set) {
            seen.add(coord);
        }
        assertEquals(List.of(a), seen, "set iteration");

        CoordSet lhs = new CoordSet();
        lhs.insert(a);
        CoordSet rhs = new CoordSet();
        rhs.insert(a);
        assertEquals(lhs, rhs, "set equality");
        rhs.insert(b);
        assertNotEquals(lhs, rhs, "set inequality");
    }

    @Test
    void fillAndRemoveAll() {
        CoordSet full = new CoordSet();
        for (int i = 0; i < Coord.N_VALID; i++) {
            full.insert(coordIndex(i));
        }
        assertEquals(Coord.N_VALID, full.size(), "set fill all size");
        assertFalse(full.isEmpty(), "set fill all not empty");
        for (int i = 0; i < Coord.N_VALID; i++) {
            assertTrue(full.contains(coordIndex(i)), "set fill all contains every coord");
        }
        for (int i = 0; i < Coord.N_VALID; i++) {
            full.remove(coordIndex(i));
        }
        assertTrue(full.isEmpty() && full.size() == 0, "set remove all");
    }

    @Test
    void fromIteratorDeduplicates() {
        List<Coord> input = Arrays.asList(coordIndex(1), coordIndex(2), coordIndex(1));
        CoordSet set = new CoordSet();
        int inserted = 0;
        for (Coord c : input) {
            if (set.insert(c)) {
                inserted += 1;
            }
        }
        assertEquals(2, inserted);
        assertEquals(2, set.size());
        assertTrue(set.contains(coordIndex(1)) && set.contains(coordIndex(2)), "set contents");
    }

    @Test
    void retainAllAndNone() {
        CoordSet set = new CoordSet();
        set.insert(coordIndex(1));
        set.insert(coordIndex(2));
        set.retain(coord -> true);
        assertEquals(2, set.size(), "set retain all keeps everything");
        set.retain(coord -> false);
        assertTrue(set.isEmpty(), "set retain none empties");
    }

    @Test
    void copyEquality() {
        CoordSet original = new CoordSet();
        original.insert(coordIndex(1));
        CoordSet copy = original.copy();
        assertEquals(copy, original);
        assertTrue(copy.contains(coordIndex(1)), "set copy contents");
    }

    @Test
    void emptySetIteration() {
        CoordSet full = new CoordSet();
        int iterated = 0;
        for (Coord ignored : full) {
            iterated += 1;
        }
        assertEquals(0, iterated);
    }
}
