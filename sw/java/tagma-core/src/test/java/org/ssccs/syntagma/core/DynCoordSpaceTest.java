package org.ssccs.syntagma.core;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.List;
import java.util.Map;
import java.util.Optional;
import org.junit.jupiter.api.Test;

/**
 * Translation of the Rust suite {@code sw/rust/core/tests/dyn_coord_space.rs}.
 * The C++ suite {@code sw/cpp/tagma_core/tests} has no DynCoordSpace test file,
 * so every case here comes from the Rust reference.
 */
class DynCoordSpaceTest {

    private static Coord coord(int index) {
        return Coord.fromIndex(index).orElseThrow();
    }

    private static List<Coord> path(int... indices) {
        Coord[] coords = new Coord[indices.length];
        for (int i = 0; i < indices.length; i++) {
            coords[i] = coord(indices[i]);
        }
        return List.of(coords);
    }

    @Test
    void emptySpaceHasNoEntries() {
        DynCoordSpace<Integer> space = new DynCoordSpace<>();
        assertEquals(Optional.empty(), space.at(path(0)), "empty space lookup");
        assertEquals(0, space.entryCount(), "empty space entry count");
        assertTrue(space.entries().isEmpty(), "empty space iteration");
    }

    @Test
    void depthOne() {
        DynCoordSpace<Integer> space = new DynCoordSpace<>();
        assertEquals(Optional.empty(), space.place(path(42), 7), "depth 1 first place");
        assertEquals(Optional.of(7), space.at(path(42)), "depth 1 lookup");
        assertEquals(1, space.entryCount(), "depth 1 entry count");
    }

    @Test
    void depthTwo() {
        DynCoordSpace<Integer> space = new DynCoordSpace<>();
        space.place(path(0, 1), 42);
        assertEquals(Optional.of(42), space.at(path(0, 1)), "depth 2 lookup");
    }

    @Test
    void depthThree() {
        DynCoordSpace<Integer> space = new DynCoordSpace<>();
        space.place(path(0, 1, 2), 99);
        assertEquals(Optional.of(99), space.at(path(0, 1, 2)), "depth 3 lookup");
    }

    @Test
    void independentPaths() {
        DynCoordSpace<Integer> space = new DynCoordSpace<>();
        space.place(path(0, 0), 10);
        space.place(path(0, 1), 20);
        assertEquals(Optional.of(10), space.at(path(0, 0)), "independent left path");
        assertEquals(Optional.of(20), space.at(path(0, 1)), "independent right path");
    }

    @Test
    void overwriteReturnsPrevious() {
        DynCoordSpace<Integer> space = new DynCoordSpace<>();
        space.place(path(5), 1);
        assertEquals(Optional.of(1), space.place(path(5), 2), "overwrite returns previous");
        assertEquals(Optional.of(2), space.at(path(5)), "overwrite stores new value");
        assertEquals(1, space.entryCount(), "overwrite keeps the entry count");
    }

    @Test
    void vacateRemoves() {
        DynCoordSpace<Integer> space = new DynCoordSpace<>();
        space.place(path(0, 1), 42);
        assertEquals(Optional.of(42), space.vacate(path(0, 1)), "vacate returns the value");
        assertEquals(Optional.empty(), space.at(path(0, 1)), "vacate empties the path");
        assertEquals(Optional.empty(), space.vacate(path(0, 1)), "vacate of an absent path");
    }

    @Test
    void mixedDepths() {
        DynCoordSpace<Integer> space = new DynCoordSpace<>();
        space.place(path(1), 10);
        space.place(path(1, 2, 3), 30);
        assertEquals(Optional.of(30), space.at(path(1, 2, 3)), "deeper path lookup");
        assertEquals(Optional.of(10), space.at(path(1)), "shorter path lookup");
        assertEquals(2, space.entryCount(), "mixed depth entry count");
    }

    @Test
    void boundaryIndices() {
        DynCoordSpace<Integer> space = new DynCoordSpace<>();
        space.place(List.of(coord(0), coord(11171)), 42);
        assertEquals(Optional.of(42), space.at(List.of(coord(0), coord(11171))), "boundary lookup");
    }

    @Test
    void prefixValueCoexistsWithDeeperPaths() {
        DynCoordSpace<Integer> space = new DynCoordSpace<>();
        space.place(path(0), 1);
        space.place(path(0, 1), 2);
        assertEquals(Optional.of(1), space.at(path(0)), "leaf is promoted to both, value kept");
        assertEquals(Optional.of(2), space.at(path(0, 1)), "deeper path kept");
        assertEquals(2, space.entryCount(), "both slot counts twice");
        assertEquals(Optional.of(1), space.vacate(path(0)), "vacate of the prefix value");
        assertEquals(Optional.of(2), space.at(path(0, 1)), "prefix removal preserves deeper paths");
    }

    @Test
    void prefixInsertAfterDeeperPath() {
        DynCoordSpace<Integer> space = new DynCoordSpace<>();
        space.place(path(0, 1), 2);
        space.place(path(0), 1);
        assertEquals(Optional.of(1), space.at(path(0)), "node is promoted to both, prefix value stored");
        assertEquals(Optional.of(2), space.at(path(0, 1)), "deeper path survives the promotion");
        assertEquals(2, space.entryCount(), "entry count after promotion");
        assertEquals(Optional.of(2), space.vacate(path(0, 1)), "vacate the deeper value");
        assertEquals(Optional.of(1), space.at(path(0)), "prefix value survives the deeper removal");
    }

    @Test
    void clearRemovesAll() {
        DynCoordSpace<Integer> space = new DynCoordSpace<>();
        space.place(path(0), 1);
        space.place(path(1, 0), 2);
        space.clear();
        assertEquals(Optional.empty(), space.at(path(0)), "cleared depth 1 path");
        assertEquals(Optional.empty(), space.at(path(1, 0)), "cleared depth 2 path");
        assertEquals(0, space.entryCount(), "cleared entry count");
    }

    @Test
    void missingPathIsAbsent() {
        DynCoordSpace<Integer> space = new DynCoordSpace<>();
        assertEquals(Optional.empty(), space.at(path(0, 0)), "missing path");
    }

    @Test
    void emptyPathLookupsAreAbsent() {
        DynCoordSpace<Integer> space = new DynCoordSpace<>();
        assertEquals(Optional.empty(), space.at(List.of()), "empty path lookup");
        assertEquals(Optional.empty(), space.vacate(List.of()), "empty path vacate");
    }

    @Test
    void emptyPathInsertRejected() {
        DynCoordSpace<Integer> space = new DynCoordSpace<>();
        IllegalArgumentException error = assertThrows(IllegalArgumentException.class,
                () -> space.place(List.of(), 42), "empty path insert");
        assertTrue(error.getMessage().contains("path must not be empty"), "empty path message");
    }

    @Test
    void copyIsIndependent() {
        DynCoordSpace<Integer> a = new DynCoordSpace<>();
        a.place(path(0), 1);
        a.place(path(1, 2), 2);
        DynCoordSpace<Integer> b = a.copy();
        b.place(path(3), 3);
        assertEquals(2, a.entryCount(), "clone source entry count");
        assertEquals(3, b.entryCount(), "clone target entry count");
        assertEquals(Optional.of(1), a.at(path(0)), "clone source shallow path");
        assertEquals(Optional.of(2), a.at(path(1, 2)), "clone source deep path");
        assertEquals(Optional.of(3), b.at(path(3)), "clone target addition");
    }

    @Test
    void entriesYieldAllPairs() {
        DynCoordSpace<Integer> space = new DynCoordSpace<>();
        space.place(path(0), 10);
        space.place(path(1, 2), 20);
        space.place(path(1, 3), 30);
        List<Map.Entry<List<Coord>, Integer>> entries = space.entries();
        assertEquals(3, entries.size(), "entries size");
        assertEquals(path(0), entries.get(0).getKey(), "entries order first");
        assertEquals(10, entries.get(0).getValue(), "entries order first value");
        assertEquals(path(1, 2), entries.get(1).getKey(), "entries order second");
        assertEquals(20, entries.get(1).getValue(), "entries order second value");
        assertEquals(path(1, 3), entries.get(2).getKey(), "entries order third");
        assertEquals(30, entries.get(2).getValue(), "entries order third value");
        for (Map.Entry<List<Coord>, Integer> entry : entries) {
            assertEquals(Optional.of(entry.getValue()), space.at(entry.getKey()),
                    "entries path matches at");
        }
    }

    @Test
    void argumentsValidated() {
        DynCoordSpace<Integer> space = new DynCoordSpace<>();
        assertThrows(NullPointerException.class, () -> space.place(path(0), null), "null value");
        assertThrows(NullPointerException.class, () -> space.at(null), "null path");
        assertThrows(NullPointerException.class, () -> space.at(java.util.Arrays.asList(null, coord(0))),
                "null coordinate");
    }
}
