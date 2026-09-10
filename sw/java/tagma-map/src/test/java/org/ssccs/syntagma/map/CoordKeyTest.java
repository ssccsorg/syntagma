package org.ssccs.syntagma.map;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.HashSet;
import java.util.Set;

import org.junit.jupiter.api.Test;
import org.ssccs.syntagma.core.Coord;
import org.ssccs.syntagma.core.CoordPath;

/**
 * Translation of the {@code CoordKey} cases of the C++ test suite
 * {@code sw/cpp/tagma_map/tests/test_map.cpp} ({@code test_coord_key},
 * {@code test_coord_key_roundtrip}, {@code test_coord_key_byte_domain}),
 * extended with the Rust-only cases of {@code sw/rust/map/src/coord_gen.rs}
 * ({@code fixed_key_eq_hash}, and the byte-domain rejection of the
 * {@code from_coord_path} documentation).
 */
class CoordKeyTest {

    @Test
    void coordKey() {
        CoordKey key = new CoordKey(new byte[] {'h', 'i'});
        assertArrayEquals(new byte[] {'h', 'i'}, key.bytes(), "coord key bytes");
        assertEquals(2, key.length(), "coord key length");
        CoordPath path = key.toCoordPath();
        assertEquals('h', path.get(0).orElseThrow().index(), "coord key path first");
        assertEquals('i', path.get(1).orElseThrow().index(), "coord key path second");
        assertEquals(key, CoordKey.fromCoordPath(path), "coord key from path");
        assertEquals(key, CoordKey.fromString("hi", 2), "coord key from string");
    }

    @Test
    void coordKeyFromIndices() {
        assertEquals(new CoordKey(new byte[] {'h', 'i'}), CoordKey.fromIndices(new int[] {104, 105}),
                "coord key from indices");
        assertEquals(new CoordKey(new byte[] {(byte) 255}), CoordKey.fromIndices(new int[] {255}),
                "coord key accepts byte domain top");
        assertThrows(IllegalArgumentException.class, () -> CoordKey.fromIndices(new int[] {0, 256}),
                "coord key rejects index at the byte domain");
        assertThrows(IllegalArgumentException.class, () -> CoordKey.fromIndices(new int[] {-1}),
                "coord key rejects negative index");
    }

    @Test
    void coordKeyWrongLengthThrows() {
        assertThrows(IllegalArgumentException.class, () -> CoordKey.fromString("hello", 2),
                "coord key wrong length throws");
    }

    @Test
    void coordKeyRoundtrip() {
        CoordKey key = new CoordKey(new byte[] {1, (byte) 255, 42});
        assertEquals(key, CoordKey.fromCoordPath(key.toCoordPath()), "coord key roundtrip");

        // Distinct byte keys map to distinct paths.
        CoordKey k1 = new CoordKey(new byte[] {0, 1});
        CoordKey k2 = new CoordKey(new byte[] {1, 0});
        assertNotEquals(k1.toCoordPath(), k2.toCoordPath(), "coord key injective");
    }

    @Test
    void coordKeyByteDomain() {
        // Inside the domain: index 255 is representable.
        CoordPath boundary = CoordPath.fromArray(coord(255));
        assertEquals(new CoordKey(new byte[] {(byte) 255}), CoordKey.fromCoordPath(boundary),
                "coord key accepts index 255");

        // At or above the domain: rejected instead of truncated.
        CoordPath high = CoordPath.fromArray(coord(256));
        IllegalArgumentException error = assertThrows(IllegalArgumentException.class,
                () -> CoordKey.fromCoordPath(high), "coord key rejects index 256");
        assertTrue(error.getMessage().contains("exceeds the byte-space domain"), error.getMessage());
    }

    @Test
    void coordKeyEqualsAndHashCode() {
        CoordKey a = new CoordKey(new byte[] {'a', 'b'});
        CoordKey b = new CoordKey(new byte[] {'a', 'b'});
        CoordKey c = new CoordKey(new byte[] {'a', 'c'});
        assertEquals(a, b, "equal keys");
        assertEquals(a.hashCode(), b.hashCode(), "equal keys share a hash");
        assertNotEquals(a, c, "different keys");

        Set<CoordKey> keys = new HashSet<>();
        keys.add(a);
        keys.add(b);
        keys.add(c);
        assertEquals(2, keys.size(), "duplicate key collapses in a set");
    }

    @Test
    void coordKeyCopiesItsBytes() {
        byte[] input = {1, 2};
        CoordKey key = new CoordKey(input);
        input[0] = 9;
        assertArrayEquals(new byte[] {1, 2}, key.bytes(), "constructor copies");

        byte[] read = key.bytes();
        read[0] = 9;
        assertArrayEquals(new byte[] {1, 2}, key.bytes(), "accessor copies");
        assertTrue(new CoordKey(new byte[0]).isEmpty(), "degenerate empty key");
    }

    private static Coord coord(int index) {
        return Coord.fromIndex(index).orElseThrow();
    }
}
