package org.ssccs.syntagma.map;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import java.util.List;
import java.util.Map;
import java.util.Optional;

import org.junit.jupiter.api.Test;
import org.ssccs.syntagma.core.Coord;
import org.ssccs.syntagma.core.CoordPath;
import org.ssccs.syntagma.core.CoordSpaceN;

/**
 * Translation of the {@code CoordMapN} cases of the C++ test suite
 * {@code sw/cpp/tagma_map/tests/test_map.cpp} ({@code test_coord_mapn},
 * {@code test_mapn_iter}), extended with the Rust-only cases of
 * {@code sw/rust/map/tests/basic.rs} ({@code mapn_insert_returns_previous},
 * {@code mapn_contains_key}, {@code mapn_wrong_length},
 * {@code mapn_by_coordkey}) and with the Java runtime checks that replace the
 * const generic {@code N} of the references: the path lookup surface reports a
 * path the store cannot hold as absent, while the coordinate-key surface and
 * the space itself reject the mismatch. The Rust {@code Default} cases have no
 * Java counterpart: the constructor is the empty-state entry point.
 */
class CoordMapNTest {

    @Test
    void coordMapN() {
        CoordMapN map = new CoordMapN(2);
        assertTrue(map.isEmpty(), "mapn initially empty");
        assertEquals(0, map.len(), "mapn initially empty length");

        assertTrue(map.insert("hi", bytes("v1")).isEmpty(), "mapn first insert");
        assertEquals(1, map.len(), "mapn length");
        assertTrue(map.containsKey("hi"), "mapn contains");
        assertValue(bytes("v1"), map.get("hi"), "mapn get");
        assertTrue(map.get("no").isEmpty(), "mapn get absent");

        assertValue(bytes("v1"), map.insert("hi", bytes("v2")), "mapn replace returns previous");
        assertEquals(1, map.len(), "mapn length after replace");

        assertValue(bytes("v2"), map.remove("hi"), "mapn remove");
        assertTrue(map.remove("hi").isEmpty(), "mapn remove again is absent");
        assertTrue(map.isEmpty(), "mapn empty after remove");
        assertTrue(map.get("hi").isEmpty(), "mapn get after remove");

        // Wrong-length keys: get/remove return empty, insert throws, mirroring
        // the references (get/remove guard on the length; insert panics or
        // throws through the key construction).
        assertTrue(map.get("x").isEmpty(), "mapn wrong length get");
        assertTrue(map.remove("x").isEmpty(), "mapn wrong length remove");
        assertThrows(IllegalArgumentException.class, () -> map.insert("x", bytes("v")),
                "mapn wrong length insert throws");

        // CoordKey API.
        CoordKey key = new CoordKey(new byte[] {'a', 'b'});
        assertTrue(map.insertByCoordKey(key, bytes("v3")).isEmpty(), "mapn insert by coordkey");
        assertTrue(map.containsKeyByCoordKey(key), "mapn contains by coordkey");
        assertValue(bytes("v3"), map.getByCoordKey(key), "mapn get by coordkey");
        assertValue(bytes("v3"), map.removeByCoordKey(key), "mapn remove by coordkey");
        assertTrue(map.isEmpty(), "mapn empty after coordkey remove");

        map.insert("hi", bytes("v1"));
        map.clear();
        assertTrue(map.isEmpty(), "mapn clear empty");
        assertEquals(0, map.len(), "mapn clear length");
    }

    @Test
    void wrongDepthKeys() {
        CoordMapN map = new CoordMapN(2);

        // The lookup surface reports a path it cannot hold as absent, so a
        // query whose center has the wrong length yields no hits instead of an
        // exception raised from inside the query loop.
        assertTrue(map.getByCoordPath(path(1, 2, 3)).isEmpty(), "long path is absent");
        assertTrue(map.getByCoordPath(path(1)).isEmpty(), "short path is absent");

        // The CoordKey surface delegates straight to the space and keeps its
        // rejection, the references' compile-time check made runtime.
        assertThrows(IllegalArgumentException.class,
                () -> map.getByCoordKey(new CoordKey(new byte[] {'a'})),
                "coordkey depth mismatch get");
        assertThrows(IllegalArgumentException.class,
                () -> map.insertByCoordKey(new CoordKey(new byte[] {'a', 'b', 'c'}), bytes("v")),
                "coordkey depth mismatch insert");
        assertThrows(IllegalArgumentException.class,
                () -> map.removeByCoordKey(new CoordKey(new byte[] {'a', 'b', 'c'})),
                "coordkey depth mismatch remove");
        assertThrows(IllegalArgumentException.class,
                () -> new CoordSpaceN<byte[]>(2).atPath(path(1, 2, 3)),
                "direct space lookup still rejects the mismatch");
    }

    @Test
    void depthValidated() {
        assertThrows(IllegalArgumentException.class, () -> new CoordMapN(0), "depth zero rejected");
    }

    @Test
    void insertReturnsPrevious() {
        CoordMapN map = new CoordMapN(3);
        assertTrue(map.insert("foo", bytes("v1")).isEmpty(), "mapn first insert");
        assertValue(bytes("v1"), map.insert("foo", bytes("v2")), "mapn second insert");
        assertEquals(1, map.len(), "mapn single entry after overwrite");
    }

    @Test
    void wrongLengthKeys() {
        CoordMapN map = new CoordMapN(3);
        assertTrue(map.get("ab").isEmpty(), "mapn short key");
        assertTrue(map.get("abcd").isEmpty(), "mapn long key");
    }

    @Test
    void containsKey() {
        CoordMapN map = new CoordMapN(3);
        assertFalse(map.containsKey("foo"), "mapn absent key");
        map.insert("foo", bytes("bar"));
        assertTrue(map.containsKey("foo"), "mapn present key");
    }

    @Test
    void byCoordKey() {
        CoordMapN map = new CoordMapN(3);
        CoordKey key = new CoordKey(new byte[] {'f', 'o', 'o'});
        map.insertByCoordKey(key, bytes("bar"));
        assertValue(bytes("bar"), map.getByCoordKey(key), "mapn by coordkey");
    }

    @Test
    void iter() {
        CoordMapN empty = new CoordMapN(2);
        assertTrue(empty.iter().isEmpty(), "mapn iter empty");

        CoordMapN map = new CoordMapN(2);
        map.insert("aa", bytes("1"));
        map.insert("bb", bytes("2"));
        List<Map.Entry<CoordKey, byte[]>> entries = map.iter();
        assertEquals(2, entries.size(), "mapn iter size");
        assertEquals(new CoordKey(new byte[] {'a', 'a'}), entries.get(0).getKey(),
                "mapn iter first in ascending order");
        assertArrayEquals(bytes("1"), entries.get(0).getValue(), "mapn iter first value");
        assertEquals(new CoordKey(new byte[] {'b', 'b'}), entries.get(1).getKey(),
                "mapn iter second in ascending order");
        assertArrayEquals(bytes("2"), entries.get(1).getValue(), "mapn iter second value");
    }

    @Test
    void valuesAreCopied() {
        CoordMapN map = new CoordMapN(2);
        byte[] value = bytes("v1");
        map.insert("hi", value);
        value[0] = 'X';
        assertValue(bytes("v1"), map.get("hi"), "insert copies the value");

        byte[] read = map.get("hi").orElseThrow();
        read[0] = 'Y';
        assertValue(bytes("v1"), map.get("hi"), "get copies the value");
    }

    private static byte[] bytes(String text) {
        return text.getBytes(StandardCharsets.UTF_8);
    }

    private static CoordPath path(int... indices) {
        Coord[] coords = new Coord[indices.length];
        for (int i = 0; i < indices.length; i++) {
            coords[i] = Coord.fromIndex(indices[i]).orElseThrow();
        }
        return CoordPath.fromArray(coords);
    }

    private static void assertValue(byte[] expected, Optional<byte[]> actual, String message) {
        assertTrue(actual.isPresent(), message + ": value present");
        assertArrayEquals(expected, actual.orElseThrow(), message);
    }
}
