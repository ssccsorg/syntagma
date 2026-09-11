package org.ssccs.syntagma.core;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.io.IOException;
import java.io.InputStream;
import java.lang.reflect.Constructor;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import org.junit.jupiter.api.Test;

/**
 * Translation of the C++ suite
 * {@code sw/cpp/tagma_core/tests/test_coord_space_m.cpp} and of the Rust suite
 * {@code sw/rust/core/tests/coord_space_m.rs}, with the file lifecycle replaced
 * by the anonymous off-heap guarantees of the Java port.
 *
 * <p>The path fixtures stay inside the first window of the space. The
 * references reserve the whole slot region with one anonymous mapping and pay
 * only for the pages they write, so a path like {@code (i, i + 1, i + 2)}
 * crosses windows for free there; the Java port materializes a whole window per
 * touched window, so the inherited fixtures would need tens of gigabytes of
 * direct memory. The slot semantics under test are the same, only the addresses
 * are confined.
 */
class CoordSpaceMTest {

    private static Coord coord(int index) {
        return Coord.fromIndex(index).orElseThrow();
    }

    private static CoordPath path3(int a, int b, int c) {
        return CoordPath.fromArray(coord(a), coord(b), coord(c));
    }

    /**
     * The slot count derived in the test from the axis ranges, so the capacity
     * assertions do not re-read {@link Coord#N_VALID}, which is the constant
     * they check.
     */
    private static long latticeSlots() {
        long characters = (long) Coord.INITIAL_MAX * Coord.MEDIAL_MAX * Coord.FINAL_MAX;
        return characters * characters * characters;
    }

    @Test
    void placeAndAt() {
        try (CoordSpaceM<Integer> space = new CoordSpaceM<>(3, Integer.class)) {
            assertTrue(space.isEmpty(), "new space is empty");
            assertEquals(0L, space.size(), "new space len is zero");
            assertEquals(latticeSlots(), space.capacity(), "fixed capacity");
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
            assertEquals(1, space.allocatedWindows(), "both values share the first window");
            space.clear();
            assertEquals(0L, space.size(), "len after clear");
            assertEquals(0, space.allocatedWindows(), "clear drops the windows");
            assertTrue(space.atPath(path3(1, 1, 1)).isEmpty(), "cleared slot one");
            assertTrue(space.atPath(path3(2, 2, 2)).isEmpty(), "cleared slot two");
            assertEquals(latticeSlots(), space.capacity(), "capacity survives clear");
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
            assertEquals(0, space.allocatedWindows(), "the constructor allocates no window");
        }
    }

    @Test
    void multipleCoords() {
        try (CoordSpaceM<Integer> space = new CoordSpaceM<>(3, Integer.class)) {
            for (int i = 0; i < 100; i++) {
                space.placePath(path3(0, 0, i), i);
            }
            assertEquals(100L, space.size(), "len after many places");
            assertEquals(1, space.allocatedWindows(), "the paths share the first window");
            for (int i = 0; i < 100; i++) {
                assertEquals(Optional.of(i), space.atPath(path3(0, 0, i)), "value at index " + i);
            }
        }
    }

    @Test
    void valuesCoverFixedWidthTypes() {
        CoordPath p = path3(1, 1, 1);
        try (CoordSpaceM<Short> space = new CoordSpaceM<>(3, Short.class)) {
            assertEquals(Optional.empty(), space.placePath(p, (short) -7), "short slot place");
            assertEquals(Optional.of((short) -7), space.atPath(p), "short slot value");
        }
        try (CoordSpaceM<Long> space = new CoordSpaceM<>(3, Long.class)) {
            space.placePath(p, 1L << 40);
            assertEquals(Optional.of(1L << 40), space.atPath(p), "long slot value");
        }
        try (CoordSpaceM<Double> space = new CoordSpaceM<>(3, Double.class)) {
            space.placePath(p, Math.PI);
            assertEquals(Optional.of(Math.PI), space.atPath(p), "double slot value");
        }
        try (CoordSpaceM<Boolean> space = new CoordSpaceM<>(3, Boolean.class)) {
            space.placePath(p, false);
            assertEquals(Optional.of(false), space.atPath(p), "false payload is not the vacant marker");
            space.placePath(p, true);
            assertEquals(Optional.of(true), space.atPath(p), "true payload");
        }
    }

    @Test
    void untouchedRegionReadsVacant() {
        try (CoordSpaceM<Integer> space = new CoordSpaceM<>(3, Integer.class)) {
            assertTrue(space.atPath(path3(0, 0, 0)).isEmpty(), "untouched first slot");
            assertTrue(space.atPath(path3(5000, 5000, 5000)).isEmpty(), "untouched middle slot");
            assertTrue(space.atPath(path3(11171, 11171, 11171)).isEmpty(), "untouched last slot");
            assertTrue(space.vacatePath(path3(0, 0, 0)).isEmpty(), "vacate of an untouched slot");
            assertEquals(0, space.allocatedWindows(), "reads and failed vacates materialize nothing");
            assertTrue(space.isEmpty(), "nothing was placed");

            space.placePath(path3(0, 0, 0), 1);
            space.placePath(path3(0, 0, 1), 2);
            space.retain((path, value) -> value == 2);
            assertEquals(1L, space.size(), "retain filters a materialized window");
            assertTrue(space.atPath(path3(0, 0, 0)).isEmpty(), "the failing value is gone");
            assertEquals(Optional.of(2), space.atPath(path3(0, 0, 1)), "the matching value stays");
            assertTrue(space.atPath(path3(11171, 11171, 11171)).isEmpty(), "the last region is still vacant");
            assertEquals(1, space.allocatedWindows(), "retain materializes no additional window");
            assertEquals(latticeSlots(), space.capacity(), "capacity is independent of allocation");
        }
    }

    @Test
    void windowMaterializedOnFirstTouch() {
        try (CoordSpaceM<Integer> space = new CoordSpaceM<>(3, Integer.class)) {
            assertEquals(0, space.allocatedWindows(), "a fresh space materializes nothing");
            space.placePath(path3(0, 0, 1), 1);
            assertEquals(1, space.allocatedWindows(), "the first placement materializes its window");
            space.placePath(path3(0, 1, 0), 2);
            assertEquals(1, space.allocatedWindows(), "a placement in a materialized window adds none");
            assertTrue(space.atPath(path3(3000, 0, 0)).isEmpty(), "a distant read finds a vacant region");
            assertEquals(1, space.allocatedWindows(), "reads never materialize a window");
            space.vacatePath(path3(0, 0, 1));
            assertEquals(1, space.allocatedWindows(), "vacate keeps the window materialized");
            space.clear();
            assertEquals(0, space.allocatedWindows(), "clear drops every window");
            space.placePath(path3(0, 0, 1), 3);
            assertEquals(1, space.allocatedWindows(), "a cleared space materializes again");
        }
    }

    @Test
    void lastWindowHoldsValues() {
        try (CoordSpaceM<Integer> space = new CoordSpaceM<>(3, Integer.class)) {
            CoordPath last = path3(Coord.N_VALID - 1, Coord.N_VALID - 1, Coord.N_VALID - 1);
            assertEquals(Optional.empty(), space.placePath(last, 7), "place in the partial last window");
            assertEquals(1, space.allocatedWindows(), "the last window materializes on first touch");
            assertEquals(Optional.of(7), space.atPath(last), "value at the last slot");
            assertEquals(1L, space.size(), "len after the distant placement");
            assertTrue(space.atPath(path3(5000, 5000, 5000)).isEmpty(), "a window in between stays vacant");
            assertEquals(1, space.allocatedWindows(), "reading a window in between materializes nothing");

            List<CoordPath> visited = new ArrayList<>();
            space.retain((path, value) -> {
                visited.add(path);
                return true;
            });
            assertEquals(List.of(last), visited, "retain walks only materialized windows");
            assertEquals(1L, space.size(), "retain kept the value");

            assertEquals(Optional.of(7), space.vacatePath(last), "vacate at the last slot");
            assertTrue(space.isEmpty(), "empty after vacating the distant value");
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
    void closeDropsWindowsAndRejectsAccess() {
        CoordSpaceM<Integer> space = new CoordSpaceM<>(3, Integer.class);
        CoordPath p = path3(1, 1, 1);
        space.placePath(p, 1);
        assertEquals(1, space.allocatedWindows(), "materialized before close");

        space.close();
        assertEquals(0, space.allocatedWindows(), "close drops the window references");
        assertEquals(0L, space.size(), "a closed space reports length zero");
        assertTrue(space.isEmpty(), "a closed space reports empty");
        assertThrows(IllegalStateException.class, () -> space.atPath(p), "atPath after close");
        assertThrows(IllegalStateException.class, () -> space.placePath(p, 1), "placePath after close");
        assertThrows(IllegalStateException.class, () -> space.vacatePath(p), "vacatePath after close");
        assertThrows(IllegalStateException.class, space::clear, "clear after close");
        assertThrows(IllegalStateException.class, () -> space.retain((path, value) -> true),
                "retain after close");

        space.close();
        assertEquals(0L, space.size(), "closing twice is harmless");
        assertEquals(3, space.depth(), "shape accessors survive close");
        assertEquals(latticeSlots(), space.capacity(), "capacity survives close");
    }

    /**
     * The space is anonymous off-heap memory, so the compiled class must not
     * reach a file API. The check reads the class file, because watching the
     * shared temporary directory is unreliable: other processes add and remove
     * entries while a test observes it.
     */
    @Test
    void classReferencesNoFileApi() throws IOException {
        byte[] bytecode;
        try (InputStream in = CoordSpaceM.class.getResourceAsStream("CoordSpaceM.class")) {
            assertNotNull(in, "the class file is on the test class path");
            bytecode = in.readAllBytes();
        }
        String pool = new String(bytecode, StandardCharsets.ISO_8859_1);
        for (String forbidden : List.of("java/nio/file/", "java/io/File", "java/nio/channels/",
                "MappedByteBuffer")) {
            assertFalse(pool.contains(forbidden), "no file api reference in the class file: " + forbidden);
        }

        try (CoordSpaceM<Integer> space = new CoordSpaceM<>(3, Integer.class)) {
            space.placePath(path3(1, 2, 3), 42);
            assertEquals(Optional.of(42), space.atPath(path3(1, 2, 3)), "the space works without a file");
            space.clear();
            space.placePath(path3(1, 2, 3), 43);
        }
    }

    @Test
    void noFileOrFormatSurfaceExists() {
        Constructor<?>[] constructors = CoordSpaceM.class.getConstructors();
        assertEquals(1, constructors.length, "exactly one public constructor");
        assertEquals(List.of(int.class, Class.class), List.of(constructors[0].getParameterTypes()),
                "the constructor takes the depth and the value type token only");
        assertThrows(NoSuchMethodException.class, () -> CoordSpaceM.class.getMethod("file"),
                "no file accessor");
        assertThrows(NoSuchMethodException.class, () -> CoordSpaceM.class.getMethod("force"),
                "no durability operation");
        assertTrue(AutoCloseable.class.isAssignableFrom(CoordSpaceM.class), "the space stays closable");
    }

    @Test
    void debugFormat() {
        try (CoordSpaceM<Integer> space = new CoordSpaceM<>(3, Integer.class)) {
            String text = space.toString();
            assertTrue(text.contains("CoordSpaceM"), "debug contains the type name");
            assertTrue(text.contains("len: 0"), "debug contains the length");
            assertTrue(text.contains("closed: false"), "debug contains the open state");
            space.placePath(path3(1, 1, 1), 1);
            assertTrue(space.toString().contains("len: 1"), "debug follows the length");
        }
    }
}
