package org.ssccs.syntagma.map;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import java.util.List;
import java.util.Map;
import java.util.Optional;

import org.junit.jupiter.api.Test;
import org.ssccs.syntagma.core.Coord;
import org.ssccs.syntagma.core.CoordPath;

/**
 * Translation of the {@code DynCoordMap} cases of the C++ test suite
 * {@code sw/cpp/tagma_map/tests/test_dyn_map.cpp} ({@code test_dyn_coord_map},
 * {@code test_dyn_iter}, {@code test_dyn_long_key_roundtrip}), extended with
 * the Rust-only cases of {@code sw/rust/map/tests/basic.rs}
 * ({@code dyn_contains_key}, {@code dyn_nonexistent_key},
 * {@code dyn_roundtrip_large_key}). The {@code test_dyn_coord_space} part of
 * the C++ suite covers {@code tagma::DynCoordSpace}, a tagma-core type whose
 * cases are already translated by
 * {@code org.ssccs.syntagma.core.DynCoordSpaceTest}.
 */
class DynCoordMapTest {

    @Test
    void dynCoordMap() {
        DynCoordMap map = new DynCoordMap();
        assertTrue(map.isEmpty(), "dynmap initially empty");
        assertEquals(0, map.len(), "dynmap initially empty length");

        assertTrue(map.insert("hello", bytes("world")).isEmpty(), "dynmap insert");
        assertEquals(1, map.len(), "dynmap length");
        assertValue(bytes("world"), map.get("hello"), "dynmap get");
        assertTrue(map.containsKey("hello"), "dynmap contains");

        assertValue(bytes("world"), map.insert("hello", bytes("earth")),
                "dynmap replace returns previous");
        assertEquals(1, map.len(), "dynmap length after replace");

        // Unicode keys map through their UTF-8 bytes.
        assertTrue(map.insert("\uD55C\uAE00", bytes("v")).isEmpty(), "dynmap unicode insert");
        assertValue(bytes("v"), map.get("\uD55C\uAE00"), "dynmap unicode get");
        assertEquals(2, map.len(), "dynmap length with unicode");

        assertTrue(map.insert("", bytes("x")).isEmpty(), "dynmap empty key rejected");
        assertTrue(map.get("").isEmpty(), "dynmap empty key get");
        assertTrue(map.remove("").isEmpty(), "dynmap empty key remove");
        assertEquals(2, map.len(), "dynmap empty key leaves length");

        assertValue(bytes("earth"), map.remove("hello"), "dynmap remove");
        assertTrue(map.get("hello").isEmpty(), "dynmap removed get");
        assertEquals(1, map.len(), "dynmap length after remove");

        map.clear();
        assertTrue(map.isEmpty(), "dynmap clear empty");
        assertEquals(0, map.len(), "dynmap clear length");
    }

    @Test
    void multipleKeys() {
        DynCoordMap map = new DynCoordMap();
        map.insert("a", bytes("1"));
        map.insert("b", bytes("2"));
        map.insert("c", bytes("3"));
        assertEquals(3, map.len(), "dynmap multiple keys length");
        assertValue(bytes("1"), map.get("a"), "dynmap first value");
        assertValue(bytes("2"), map.get("b"), "dynmap second value");
        assertValue(bytes("3"), map.get("c"), "dynmap third value");
    }

    @Test
    void containsKey() {
        DynCoordMap map = new DynCoordMap();
        assertFalse(map.containsKey("hello"), "dynmap absent contains");
        map.insert("hello", bytes("world"));
        assertTrue(map.containsKey("hello"), "dynmap present contains");
    }

    @Test
    void nonexistentKey() {
        DynCoordMap map = new DynCoordMap();
        assertTrue(map.get("nonexistent").isEmpty(), "dynmap absent get");
        assertTrue(map.remove("nonexistent").isEmpty(), "dynmap absent remove");
    }

    @Test
    void iter() {
        DynCoordMap empty = new DynCoordMap();
        assertTrue(empty.iter().isEmpty(), "dyn iter empty");

        DynCoordMap map = new DynCoordMap();
        map.insert("abc", bytes("123"));
        map.insert("def", bytes("456"));
        List<Map.Entry<String, byte[]>> entries = map.iter();
        assertEquals(2, entries.size(), "dyn iter size");
        assertEquals("abc", entries.get(0).getKey(), "dyn iter first in ascending order");
        assertArrayEquals(bytes("123"), entries.get(0).getValue(), "dyn iter first value");
        assertEquals("def", entries.get(1).getKey(), "dyn iter second in ascending order");
        assertArrayEquals(bytes("456"), entries.get(1).getValue(), "dyn iter second value");
    }

    @Test
    void getByCoordPath() {
        DynCoordMap map = new DynCoordMap();
        map.insert("ab", bytes("v1"));
        assertValue(bytes("v1"), map.getByCoordPath(path(97, 98)), "dynmap lookup by path");
        assertTrue(map.getByCoordPath(path(97, 99)).isEmpty(), "dynmap absent path");
        // A coordinate index no byte key can reach is simply absent.
        assertTrue(map.getByCoordPath(path(300, 300)).isEmpty(), "dynmap path above the domain");
    }

    @Test
    void longKeyRoundtrip() {
        // 64-byte key: the byte-wise path length equals the key size and
        // lookup is O(key length). Mirrors dyn_roundtrip_large_key.
        byte[] raw = new byte[64];
        for (int i = 0; i < raw.length; i++) {
            raw[i] = (byte) i;
        }
        String longKey = new String(raw, StandardCharsets.ISO_8859_1);

        DynCoordMap map = new DynCoordMap();
        assertTrue(map.insert(longKey, bytes("v")).isEmpty(), "dyn long key insert");
        assertValue(bytes("v"), map.get(longKey), "dyn long key get");
        assertEquals(1, map.len(), "dyn long key length");
        assertEquals(64, CoordGen.stringToCoordPath(longKey).orElseThrow().size(),
                "dyn long key path length");
    }

    @Test
    void valuesAreCopied() {
        DynCoordMap map = new DynCoordMap();
        byte[] value = bytes("v1");
        map.insert("hello", value);
        value[0] = 'X';
        assertValue(bytes("v1"), map.get("hello"), "insert copies the value");

        byte[] read = map.get("hello").orElseThrow();
        read[0] = 'Y';
        assertValue(bytes("v1"), map.get("hello"), "get copies the value");
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
