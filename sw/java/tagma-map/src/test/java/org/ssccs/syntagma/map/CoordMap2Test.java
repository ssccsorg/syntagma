package org.ssccs.syntagma.map;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import java.util.Optional;

import org.junit.jupiter.api.Test;

/**
 * Translation of the {@code CoordMap2} cases of the Rust suite
 * {@code sw/rust/map/tests/basic.rs} ({@code map2_new_is_empty},
 * {@code map2_insert_and_get}, {@code map2_insert_overwrite},
 * {@code map2_remove}, {@code map2_multiple_keys},
 * {@code map2_nonexistent_key}, {@code map2_wrong_length_returns_none},
 * {@code map2_clear}, {@code map2_contains_key},
 * {@code map2_contains_key_wrong_length},
 * {@code map2_contains_key_by_coordkey}, {@code map2_by_coordkey},
 * {@code map2_by_coordkey_remove}, {@code map2_insert_returns_previous}); the
 * C++ suite {@code sw/cpp/tagma_map/tests/test_cube_map.cpp} exercises the same
 * type only through the spatial queries, which
 * {@link CoordCubeMapTest} translates. The Rust {@code Default} cases have no
 * Java counterpart: the constructor is the empty-state entry point.
 */
class CoordMap2Test {

    @Test
    void newIsEmpty() {
        CoordMap2 map = new CoordMap2();
        assertTrue(map.isEmpty(), "map2 new is empty");
        assertEquals(0, map.len(), "map2 new length");
    }

    @Test
    void insertAndGet() {
        CoordMap2 map = new CoordMap2();
        map.insert("hi", bytes("world"));
        assertValue(bytes("world"), map.get("hi"), "map2 get");
        assertEquals(1, map.len(), "map2 length");
    }

    @Test
    void insertOverwrite() {
        CoordMap2 map = new CoordMap2();
        map.insert("ky", bytes("v1"));
        map.insert("ky", bytes("v2"));
        assertValue(bytes("v2"), map.get("ky"), "map2 overwrite");
        assertEquals(1, map.len(), "map2 length after overwrite");
    }

    @Test
    void insertReturnsPrevious() {
        CoordMap2 map = new CoordMap2();
        assertTrue(map.insert("ky", bytes("v1")).isEmpty(), "map2 first insert");
        assertValue(bytes("v1"), map.insert("ky", bytes("v2")), "map2 second insert");
        assertEquals(1, map.len(), "map2 length after second insert");
    }

    @Test
    void remove() {
        CoordMap2 map = new CoordMap2();
        map.insert("ky", bytes("value"));
        assertValue(bytes("value"), map.remove("ky"), "map2 remove");
        assertTrue(map.isEmpty(), "map2 empty after remove");
        assertTrue(map.get("ky").isEmpty(), "map2 get after remove");
    }

    @Test
    void multipleKeys() {
        CoordMap2 map = new CoordMap2();
        map.insert("aa", bytes("1"));
        map.insert("bb", bytes("2"));
        map.insert("cc", bytes("3"));
        assertEquals(3, map.len(), "map2 multiple keys length");
        assertValue(bytes("1"), map.get("aa"), "map2 first value");
        assertValue(bytes("2"), map.get("bb"), "map2 second value");
        assertValue(bytes("3"), map.get("cc"), "map2 third value");
    }

    @Test
    void nonexistentKey() {
        CoordMap2 map = new CoordMap2();
        assertTrue(map.get("no").isEmpty(), "map2 absent key");
    }

    @Test
    void wrongLengthReturnsNone() {
        CoordMap2 map = new CoordMap2();
        assertTrue(map.get("hello").isEmpty(), "map2 long key absent");
        assertTrue(map.get("x").isEmpty(), "map2 short key absent");
        assertTrue(map.remove("hello").isEmpty(), "map2 long key remove absent");
        assertThrows(IllegalArgumentException.class, () -> map.insert("hello", bytes("world")),
                "map2 wrong length insert throws");
    }

    @Test
    void clear() {
        CoordMap2 map = new CoordMap2();
        map.insert("aa", bytes("1"));
        map.insert("bb", bytes("2"));
        assertEquals(2, map.len(), "map2 length before clear");
        map.clear();
        assertTrue(map.isEmpty(), "map2 empty after clear");
        assertTrue(map.get("aa").isEmpty(), "map2 cleared key absent");
    }

    @Test
    void containsKey() {
        CoordMap2 map = new CoordMap2();
        assertFalse(map.containsKey("hi"), "map2 absent contains");
        map.insert("hi", bytes("world"));
        assertTrue(map.containsKey("hi"), "map2 present contains");
    }

    @Test
    void containsKeyWrongLength() {
        CoordMap2 map = new CoordMap2();
        assertFalse(map.containsKey("hello"), "map2 wrong length contains");
    }

    @Test
    void byCoordKey() {
        CoordMap2 map = new CoordMap2();
        CoordKey key = new CoordKey(new byte[] {'h', 'i'});
        map.insertByCoordKey(key, bytes("world"));
        assertValue(bytes("world"), map.getByCoordKey(key), "map2 by coordkey");
        assertEquals(1, map.len(), "map2 by coordkey length");
    }

    @Test
    void byCoordKeyRemove() {
        CoordMap2 map = new CoordMap2();
        CoordKey key = new CoordKey(new byte[] {'k', 'y'});
        map.insertByCoordKey(key, bytes("val"));
        assertValue(bytes("val"), map.removeByCoordKey(key), "map2 by coordkey remove");
        assertTrue(map.isEmpty(), "map2 empty after by coordkey remove");
    }

    @Test
    void containsKeyByCoordKey() {
        CoordMap2 map = new CoordMap2();
        CoordKey key = new CoordKey(new byte[] {'h', 'i'});
        assertFalse(map.containsKeyByCoordKey(key), "map2 absent coordkey");
        map.insertByCoordKey(key, bytes("world"));
        assertTrue(map.containsKeyByCoordKey(key), "map2 present coordkey");
    }

    @Test
    void wrongLengthByCoordKeyRejected() {
        CoordMap2 map = new CoordMap2();
        assertThrows(IllegalArgumentException.class,
                () -> map.getByCoordKey(new CoordKey(new byte[] {'h', 'i', 'j'})),
                "map2 coordkey depth mismatch");
    }

    private static byte[] bytes(String text) {
        return text.getBytes(StandardCharsets.UTF_8);
    }

    private static void assertValue(byte[] expected, Optional<byte[]> actual, String message) {
        assertTrue(actual.isPresent(), message + ": value present");
        assertArrayEquals(expected, actual.orElseThrow(), message);
    }
}
