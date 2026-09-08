package org.ssccs.syntagma.core;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.Optional;
import org.junit.jupiter.api.Test;

/**
 * Translation of the CoordPath section of the C++ test suite
 * {@code sw/cpp/tagma_core/tests/test_core_types.cpp}.
 */
class CoordPathTest {

    private static Coord coord(int initial, int medial, int final_) {
        return Coord.fromAxes(initial, medial, final_).orElseThrow();
    }

    @Test
    void pathLengthAndContents() {
        Coord[] coords = {coord(0, 0, 0), coord(1, 2, 3), coord(18, 20, 27)};
        CoordPath path = CoordPath.fromArray(coords);
        assertEquals(3, path.length());
        assertFalse(path.isEmpty());
        assertArrayEquals(coords, path.coords());
    }

    @Test
    void pathGet() {
        Coord[] coords = {coord(0, 0, 0), coord(1, 2, 3), coord(18, 20, 27)};
        CoordPath path = CoordPath.fromArray(coords);
        assertEquals(Optional.of(coords[0]), path.get(0));
        assertEquals(Optional.of(coords[2]), path.get(2));
        assertTrue(path.get(3).isEmpty(), "path get out of range");
        assertTrue(path.get(-1).isEmpty(), "path get negative");
    }

    @Test
    void pathIteration() {
        Coord[] coords = {coord(0, 0, 0), coord(1, 2, 3), coord(18, 20, 27)};
        CoordPath path = CoordPath.fromArray(coords);
        int count = 0;
        for (Coord ignored : path) {
            count += 1;
        }
        assertEquals(3, count);
    }

    @Test
    void emptyPath() {
        CoordPath empty = CoordPath.fromArray();
        assertEquals(0, empty.length());
        assertTrue(empty.isEmpty());
    }
}
