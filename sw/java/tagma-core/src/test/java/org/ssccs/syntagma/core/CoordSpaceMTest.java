package org.ssccs.syntagma.core;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Optional;
import org.junit.jupiter.api.Test;

/**
 * Translation of the C++ suite
 * {@code sw/cpp/tagma_core/tests/test_coord_space_m.cpp}, extended with the
 * cases of the Rust suite {@code sw/rust/core/tests/coord_space_m.rs} and with
 * the file-backed behaviors that replace the Unix-only anonymous mapping.
 */
class CoordSpaceMTest {

    private static Coord coord(int index) {
        return Coord.fromIndex(index).orElseThrow();
    }

    private static CoordPath path3(int a, int b, int c) {
        return CoordPath.fromArray(coord(a), coord(b), coord(c));
    }

    @Test
    void placeAndAt() {
        try (CoordSpaceM<Integer> space = new CoordSpaceM<>(3, Integer.class)) {
            assertTrue(space.isEmpty(), "new space is empty");
            assertEquals(0L, space.size(), "new space len is zero");
            assertEquals(CoordSpaceM.SLOT_COUNT, space.capacity(), "fixed capacity");
            CoordPath p = path3(1, 2, 3);
            assertTrue(space.atPath(p).isEmpty(), "at_path on empty slot");
            assertTrue(space.placePath(p, 42).isEmpty(), "place returns no previous");
            assertEquals(1L, space.size(), "len after place");
            assertEquals(Optional.of(42), space.atPath(p), "at_path finds value");
        }
    }

    @Test
    void placeOverwrite() {
        try (CoordSpaceM<Integer> space = new CoordSpaceM<>(3, Integer.class)) {
            CoordPath p = path3(5, 5, 5);
            space.placePath(p, 1);
            assertEquals(Optional.of(1), space.placePath(p, 2), "overwrite returns previous");
            assertEquals(1L, space.size(), "len unchanged on overwrite");
            assertEquals(Optional.of(2), space.atPath(p), "overwrite stores new value");
        }
    }

    @Test
    void vacate() {
        try (CoordSpaceM<Integer> space = new CoordSpaceM<>(3, Integer.class)) {
            CoordPath p = path3(9, 9, 9);
            space.placePath(p, 7);
            assertEquals(Optional.of(7), space.vacatePath(p), "vacate returns value");
            assertTrue(space.atPath(p).isEmpty(), "vacate empties slot");
            assertEquals(0L, space.size(), "len after vacate");
            assertTrue(space.vacatePath(p).isEmpty(), "vacate on empty slot");
        }
    }

    @Test
    void clear() {
        try (CoordSpaceM<Integer> space = new CoordSpaceM<>(3, Integer.class)) {
            space.placePath(path3(1, 1, 1), 11);
            space.placePath(path3(2, 2, 2), 22);
            assertEquals(2L, space.size(), "len before clear");
            space.clear();
            assertEquals(0L, space.size(), "len after clear");
            assertTrue(space.atPath(path3(1, 1, 1)).isEmpty(), "cleared slot one");
            assertTrue(space.atPath(path3(2, 2, 2)).isEmpty(), "cleared slot two");
            assertEquals(CoordSpaceM.SLOT_COUNT, space.capacity(), "capacity survives clear");
            assertEquals(Optional.empty(), space.placePath(path3(3, 3, 3), 33), "clear leaves the space usable");
            assertEquals(Optional.of(33), space.atPath(path3(3, 3, 3)), "reuse after clear");
        }
    }

    @Test
    void distinctPathsDistinctSlots() {
        try (CoordSpaceM<Integer> space = new CoordSpaceM<>(3, Integer.class)) {
            CoordPath a = path3(0, 0, 1);
            CoordPath b = path3(0, 1, 0);
            space.placePath(a, 1);
            space.placePath(b, 2);
            assertEquals(2L, space.size(), "distinct paths occupy distinct slots");
            assertEquals(Optional.of(1), space.atPath(a), "value at path a");
            assertEquals(Optional.of(2), space.atPath(b), "value at path b");
        }
    }

    @Test
    void unsupportedDepthThrows() {
        assertThrows(IllegalArgumentException.class,
                () -> new CoordSpaceM<Integer>(6, Integer.class), "depth 6 is unsupported");
        assertThrows(IllegalArgumentException.class,
                () -> new CoordSpaceM<Integer>(0, Integer.class), "depth 0 is unsupported");
    }

    @Test
    void unsupportedValueTypeThrows() {
        assertThrows(IllegalArgumentException.class,
                () -> new CoordSpaceM<Object>(3, Object.class), "object values are unsupported");
        assertThrows(IllegalArgumentException.class,
                () -> new CoordSpaceM<String>(3, String.class), "string values are unsupported");
    }

    @Test
    void defaultIsEmpty() {
        try (CoordSpaceM<Integer> space = new CoordSpaceM<>(3, Integer.class)) {
            assertTrue(space.isEmpty(), "default constructor is empty");
            assertEquals(3, space.depth(), "default constructor depth");
            assertEquals(0L, space.size(), "default constructor length");
        }
    }

    @Test
    void multipleCoords() {
        try (CoordSpaceM<Integer> space = new CoordSpaceM<>(3, Integer.class)) {
            for (int i = 0; i < 100; i++) {
                space.placePath(path3(i, i + 1, i + 2), i);
            }
            assertEquals(100L, space.size(), "len after many places");
            for (int i = 0; i < 100; i++) {
                assertEquals(Optional.of(i), space.atPath(path3(i, i + 1, i + 2)), "value at index " + i);
            }
        }
    }

    @Test
    void valuesCoverFixedWidthTypes() {
        try (CoordSpaceM<Short> shorts = new CoordSpaceM<>(3, Short.class);
                CoordSpaceM<Long> longs = new CoordSpaceM<>(3, Long.class);
                CoordSpaceM<Double> doubles = new CoordSpaceM<>(3, Double.class);
                CoordSpaceM<Boolean> booleans = new CoordSpaceM<>(3, Boolean.class)) {
            CoordPath p = path3(1, 1, 1);
            assertEquals(Optional.empty(), shorts.placePath(p, (short) -7), "short slot place");
            assertEquals(Optional.of((short) -7), shorts.atPath(p), "short slot value");
            longs.placePath(p, 1L << 40);
            assertEquals(Optional.of(1L << 40), longs.atPath(p), "long slot value");
            doubles.placePath(p, Math.PI);
            assertEquals(Optional.of(Math.PI), doubles.atPath(p), "double slot value");
            booleans.placePath(p, false);
            assertEquals(Optional.of(false), booleans.atPath(p), "false payload is not the vacant marker");
            booleans.placePath(p, true);
            assertEquals(Optional.of(true), booleans.atPath(p), "true payload");
        }
    }

    @Test
    void distantSlotInAnotherWindow() throws IOException {
        Path file = Files.createTempFile("coord-space-m-distant", ".bin");
        try (CoordSpaceM<Integer> space = new CoordSpaceM<>(3, Integer.class, file)) {
            CoordPath last = path3(11171, 11171, 11171);
            assertEquals(Optional.empty(), space.placePath(last, 7), "place in the last window");
            space.placePath(path3(0, 0, 0), 8);
            assertEquals(2L, space.size(), "len across windows");
            assertEquals(Optional.of(7), space.atPath(last), "value in the last window");
            assertEquals(Optional.of(8), space.atPath(path3(0, 0, 0)), "value in the first window");

            space.retain((path, value) -> value == 8);
            assertEquals(1L, space.size(), "retain across windows");
            assertTrue(space.atPath(last).isEmpty(), "retain dropped the distant value");
            assertEquals(Optional.of(8), space.atPath(path3(0, 0, 0)), "retain kept the near value");

            assertTrue(space.atPath(path3(5000, 5000, 5000)).isEmpty(),
                    "a covered but never written window reads as vacant");
            assertEquals(1L, space.size(), "reading a covered window does not change the count");
        } finally {
            Files.deleteIfExists(file);
        }
    }

    @Test
    void valuesPersistAcrossReopen() throws IOException {
        Path file = Files.createTempFile("coord-space-m-persist", ".bin");
        try {
            CoordPath p = path3(10, 20, 30);
            try (CoordSpaceM<Integer> writer = new CoordSpaceM<>(3, Integer.class, file)) {
                writer.placePath(p, 42);
                writer.placePath(path3(1, 1, 1), 1);
                assertEquals(2L, writer.size(), "writer length");
            }
            try (CoordSpaceM<Integer> reader = new CoordSpaceM<>(3, Integer.class, file)) {
                assertEquals(2L, reader.size(), "reader restores the engaged count");
                assertEquals(Optional.of(42), reader.atPath(p), "reader sees the persisted value");
                assertEquals(Optional.of(1), reader.atPath(path3(1, 1, 1)), "reader sees the second value");
                assertEquals(Optional.of(42), reader.vacatePath(p), "reader can vacate");
                assertEquals(1L, reader.size(), "vacate changes the persisted count");
            }
            try (CoordSpaceM<Integer> reader = new CoordSpaceM<>(3, Integer.class, file)) {
                assertEquals(1L, reader.size(), "third open sees the vacate");
                assertTrue(reader.atPath(path3(10, 20, 30)).isEmpty(), "vacated slot stays vacant");
            }
        } finally {
            Files.deleteIfExists(file);
        }
    }

    @Test
    void clearPersistsAcrossReopen() throws IOException {
        Path file = Files.createTempFile("coord-space-m-clear", ".bin");
        try {
            try (CoordSpaceM<Integer> space = new CoordSpaceM<>(3, Integer.class, file)) {
                space.placePath(path3(1, 2, 3), 42);
                space.clear();
            }
            try (CoordSpaceM<Integer> space = new CoordSpaceM<>(3, Integer.class, file)) {
                assertTrue(space.isEmpty(), "cleared space reopens empty");
                assertEquals(0L, space.size(), "cleared space length after reopen");
                assertTrue(space.atPath(path3(1, 2, 3)).isEmpty(), "cleared slot after reopen");
            }
        } finally {
            Files.deleteIfExists(file);
        }
    }

    @Test
    void retainKeepsMatchingValues() {
        try (CoordSpaceM<Integer> space = new CoordSpaceM<>(3, Integer.class)) {
            space.placePath(path3(0, 0, 0), 3);
            space.placePath(path3(0, 0, 1), 7);
            space.placePath(path3(0, 0, 2), 9);
            space.retain((path, value) -> value > 5);
            assertEquals(2L, space.size(), "retain length");
            assertTrue(space.atPath(path3(0, 0, 0)).isEmpty(), "retain dropped the failing value");
            assertEquals(Optional.of(7), space.atPath(path3(0, 0, 1)), "retain kept the first passing value");
            assertEquals(Optional.of(9), space.atPath(path3(0, 0, 2)), "retain kept the second passing value");

            space.retain((path, value) -> path.equals(path3(0, 0, 2)));
            assertEquals(1L, space.size(), "retain by path length");
            assertEquals(Optional.of(9), space.atPath(path3(0, 0, 2)), "retain by path kept the match");

            space.retain((path, value) -> false);
            assertTrue(space.isEmpty(), "retain with a rejecting predicate empties the space");
            assertTrue(space.vacatePath(path3(0, 0, 2)).isEmpty(), "vacate after retain");
        }
    }

    @Test
    void valueTypeMismatchRejectedOnReopen() throws IOException {
        Path file = Files.createTempFile("coord-space-m-mismatch", ".bin");
        try (CoordSpaceM<Integer> space = new CoordSpaceM<>(3, Integer.class, file)) {
            space.placePath(path3(1, 1, 1), 1);
        } finally {
            assertThrows(IllegalArgumentException.class,
                    () -> new CoordSpaceM<Long>(3, Long.class, file), "value type mismatch rejected");
            Files.deleteIfExists(file);
        }
    }

    @Test
    void foreignFileRejected() throws IOException {
        Path file = Files.createTempFile("coord-space-m-foreign", ".bin");
        try {
            Files.write(file, new byte[4096]);
            assertThrows(IllegalArgumentException.class,
                    () -> new CoordSpaceM<Integer>(3, Integer.class, file), "foreign header rejected");
            Files.write(file, new byte[10]);
            assertThrows(IllegalArgumentException.class,
                    () -> new CoordSpaceM<Integer>(3, Integer.class, file), "short file rejected");
        } finally {
            Files.deleteIfExists(file);
        }
    }

    @Test
    void accessAfterCloseRejected() {
        CoordSpaceM<Integer> space = new CoordSpaceM<>(3, Integer.class);
        space.placePath(path3(1, 1, 1), 1);
        space.close();
        space.close();
        CoordPath p = path3(1, 1, 1);
        assertThrows(IllegalStateException.class, () -> space.atPath(p), "atPath after close");
        assertThrows(IllegalStateException.class, () -> space.placePath(p, 1), "placePath after close");
        assertThrows(IllegalStateException.class, () -> space.vacatePath(p), "vacatePath after close");
        assertThrows(IllegalStateException.class, space::clear, "clear after close");
        assertThrows(IllegalStateException.class, () -> space.retain((path, value) -> true), "retain after close");
        assertThrows(IllegalStateException.class, space::force, "force after close");
    }

    @Test
    void argumentsValidated() {
        try (CoordSpaceM<Integer> space = new CoordSpaceM<>(3, Integer.class)) {
            CoordPath shortPath = CoordPath.fromArray(coord(1), coord(1));
            assertThrows(IllegalArgumentException.class, () -> space.atPath(shortPath),
                    "short path rejected");
            assertThrows(NullPointerException.class, () -> space.placePath(null, 1), "null path rejected");
            assertThrows(NullPointerException.class, () -> space.placePath(path3(1, 1, 1), null),
                    "null value rejected");
        }
    }

    @Test
    void toStringMentionsShape() {
        try (CoordSpaceM<Integer> space = new CoordSpaceM<>(3, Integer.class)) {
            String text = space.toString();
            assertTrue(text.contains("CoordSpaceM"), "debug contains the type name");
            assertTrue(text.contains("len: 0"), "debug contains the length");
            space.placePath(path3(1, 1, 1), 1);
            assertTrue(space.toString().contains("len: 1"), "debug follows the length");
            assertFalse(space.file().toString().isEmpty(), "backing file is exposed");
        }
    }
}
