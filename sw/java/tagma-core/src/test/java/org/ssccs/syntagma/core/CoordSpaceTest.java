package org.ssccs.syntagma.core;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.List;
import java.util.Map;
import java.util.Optional;
import org.junit.jupiter.api.Test;

/**
 * Translation of the CoordSpace section of the C++ test suite
 * {@code sw/cpp/tagma_core/tests/test_core_types.cpp}.
 */
class CoordSpaceTest {

    private static Coord coord(int initial, int medial, int final_) {
        return Coord.fromAxes(initial, medial, final_).orElseThrow();
    }

    @Test
    void placeReplaceVacate() {
        CoordSpace<Integer> space = new CoordSpace<>();
        assertTrue(space.isEmpty(), "space initially empty");
        assertEquals(0, space.size(), "space initial size");

        Coord a = coord(0, 0, 0);
        Coord b = coord(5, 10, 15);

        assertEquals(Optional.empty(), space.place(a, 11), "first place no previous");
        assertEquals(Optional.of(11), space.place(a, 22), "replace returns previous");
        assertEquals(1, space.size(), "space size after place");
        assertTrue(space.occupied(a), "space occupied");
        assertFalse(space.occupied(b), "space not occupied at b");
        assertEquals(Optional.of(22), space.at(a), "space at");
        assertEquals(Optional.empty(), space.at(b), "space at absent");

        space.atMut(a).orElseThrow().set(33);
        assertEquals(Optional.of(33), space.at(a), "space at_mut");

        assertEquals(Optional.empty(), space.vacate(b), "vacate absent");
        assertEquals(Optional.of(33), space.vacate(a), "vacate present");
        assertTrue(space.isEmpty(), "space empty after vacate");
    }

    @Test
    void clearAndCapacity() {
        CoordSpace<Integer> space = new CoordSpace<>();
        Coord a = coord(0, 0, 0);
        Coord b = coord(5, 10, 15);
        space.place(a, 1);
        space.place(b, 2);
        space.clear();
        assertTrue(space.isEmpty() && space.size() == 0, "space clear");
        int lattice = Coord.INITIAL_MAX * Coord.MEDIAL_MAX * Coord.FINAL_MAX;
        assertEquals(lattice, CoordSpace.capacity(), "space capacity");
    }

    @Test
    void pathAccess() {
        CoordSpace<Integer> space = new CoordSpace<>();
        Coord a = coord(0, 0, 0);
        CoordPath pathA = CoordPath.fromArray(a);
        assertEquals(Optional.empty(), space.placePath(pathA, 5), "place path");
        assertEquals(Optional.of(5), space.atPath(pathA), "at path");
        assertEquals(Optional.of(5), space.vacatePath(pathA), "vacate path");
    }

    @Test
    void entryOrInsert() {
        CoordSpace<Integer> space = new CoordSpace<>();
        Coord a = coord(0, 0, 0);
        Coord b = coord(5, 10, 15);

        space.entry(a).orInsert(7);
        space.entry(a).orInsert(9);
        assertEquals(Optional.of(7), space.at(a), "entry or_insert keeps existing");
        space.entry(b).orInsert(3);
        assertEquals(Optional.of(3), space.at(b), "entry or_insert inserts");
        space.entry(b).orInsert(0).set(3);
        assertEquals(Optional.of(3), space.at(b), "entry or_insert mutable reference");
    }

    @Test
    void retainAndOrInsertWith() {
        CoordSpace<Integer> space = new CoordSpace<>();
        Coord a = coord(0, 0, 0);
        Coord b = coord(5, 10, 15);

        space.place(a, 7);
        space.place(b, 3);
        space.retain((coord, value) -> value > 5);
        assertTrue(space.occupied(a) && !space.occupied(b), "space retain");

        space.entry(b).orInsertWith(() -> 5);
        assertEquals(Optional.of(5), space.at(b), "space entry or_insert_with");
        assertEquals(2, space.size(), "space size after or_insert_with");
        space.vacate(b);
    }

    /**
     * The factory runs only for a vacant slot. The C++ {@code or_insert_with}
     * and the Rust {@code or_insert_with} evaluate it inside the vacant branch,
     * so an occupied slot must not pay for a value it discards.
     */
    @Test
    void orInsertWithEvaluatesOnlyWhenVacant() {
        CoordSpace<Integer> space = new CoordSpace<>();
        Coord a = coord(0, 0, 0);
        int[] calls = {0};

        space.entry(a).orInsertWith(() -> {
            calls[0] += 1;
            return 7;
        });
        assertEquals(1, calls[0], "the factory runs for the vacant slot");
        assertEquals(Optional.of(7), space.at(a), "the factory value is stored");

        space.entry(a).orInsertWith(() -> {
            calls[0] += 1;
            return 9;
        });
        assertEquals(1, calls[0], "the factory does not run for the occupied slot");
        assertEquals(Optional.of(7), space.at(a), "the existing value is kept");
    }

    @Test
    void iteration() {
        CoordSpace<Integer> space = new CoordSpace<>();
        Coord a = coord(0, 0, 0);
        Coord b = coord(5, 10, 15);
        space.place(a, 7);
        space.place(b, 3);
        space.retain((coord, value) -> value > 5);
        space.vacate(b);

        List<Map.Entry<Coord, Integer>> entries = space.entries();
        assertEquals(1, entries.size(), "space iteration count");
        assertEquals(a, entries.get(0).getKey(), "space iteration coord");
        assertEquals(7, entries.get(0).getValue(), "space iteration value");
    }

    /**
     * Mirrors the Rust chained-entry pattern: repeated
     * {@code *space.entry(c).or_insert(0) += 1} through a single write-through
     * handle.
     */
    @Test
    void entryChainedIncrement() {
        CoordSpace<Integer> space = new CoordSpace<>();
        Coord c = coord(0, 0, 0);
        for (int i = 0; i < 5; i++) {
            space.entry(c).orInsert(0).update(value -> value + 1);
        }
        assertEquals(Optional.of(5), space.at(c));
    }
}
