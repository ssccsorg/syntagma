package org.ssccs.syntagma.map;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertSame;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.HashSet;
import java.util.List;
import java.util.Optional;
import java.util.OptionalInt;
import java.util.Set;

import org.junit.jupiter.api.Test;
import org.ssccs.syntagma.core.Coord;

/**
 * Translation of the strategy cases of the C++ test suite
 * {@code sw/cpp/tagma_map/tests/test_map.cpp} ({@code test_bytewise},
 * {@code test_charwise}, {@code test_prefix}, {@code test_bytefold},
 * {@code test_string_to_coord_path}, {@code test_strategy_edge_cases}),
 * extended with the Rust-only cases of
 * {@code sw/rust/map/src/coord_gen.rs} (the {@code ByteWise} /
 * {@code CharWise} / {@code Prefix} / {@code ByteFold} unit tests) and with the
 * Java runtime validation of the depth parameter, which the references assert
 * at compile time.
 */
class CoordGenTest {

    @Test
    void byteWise() {
        List<Coord> coords = generate(ByteWise.INSTANCE, "hi");
        assertEquals(2, coords.size(), "bytewise path length");
        assertEquals('h', coords.get(0).index(), "bytewise first index");
        assertEquals('i', coords.get(1).index(), "bytewise second index");
        assertTrue(ByteWise.INSTANCE.generate("").isEmpty(), "bytewise empty");
        assertTrue(ByteWise.INSTANCE.isInjective(), "bytewise injective");
        assertTrue(ByteWise.INSTANCE.fixedDepth().isEmpty(), "bytewise dynamic");
        assertEquals("byte-wise", ByteWise.INSTANCE.name(), "bytewise name");

        // Unicode: UTF-8 bytes map one to one.
        assertEquals(3, generate(ByteWise.INSTANCE, "\uAC00").size(), "bytewise utf8 bytes");
        assertEquals(3, generate(ByteWise.INSTANCE, "\uD55C").size(), "bytewise hangul bytes");
    }

    @Test
    void byteWiseIsInjective() {
        // Keys that share their leading bytes: a first-byte-only or truncating
        // mapping would collapse them.
        String[] keys = {"ab", "ac", "ba", "abc"};
        Set<List<Coord>> paths = new HashSet<>();
        for (String key : keys) {
            paths.add(generate(ByteWise.INSTANCE, key));
        }
        assertEquals(keys.length, paths.size(), "different keys must produce different paths");

        // Strong form: every key survives in a store and is retrievable by its
        // own key.
        DynCoordMap map = new DynCoordMap();
        for (int i = 0; i < keys.length; i++) {
            map.insert(keys[i], new byte[] {(byte) ('1' + i)});
        }
        assertEquals(keys.length, map.len(), "shared-prefix keys coexist");
        for (int i = 0; i < keys.length; i++) {
            assertValue(new byte[] {(byte) ('1' + i)}, map.get(keys[i]), "key " + keys[i]);
        }
    }

    @Test
    void charWise() {
        List<Coord> coords = generate(CharWise.INSTANCE, "hi");
        assertEquals(4, coords.size(), "charwise two per char");
        int h = 'h';
        assertEquals(h / 11172, coords.get(0).index(), "charwise first char high part");
        assertEquals(h % 11172, coords.get(1).index(), "charwise first char low part");
        assertTrue(CharWise.INSTANCE.generate("").isEmpty(), "charwise empty");
        assertTrue(CharWise.INSTANCE.isInjective(), "charwise injective");
        assertTrue(CharWise.INSTANCE.fixedDepth().isEmpty(), "charwise dynamic");
        assertEquals("char-wise", CharWise.INSTANCE.name(), "charwise name");

        // Rust charwise_basic: "ab" -> [0, 97, 0, 98].
        List<Coord> ascii = generate(CharWise.INSTANCE, "ab");
        assertEquals(List.of(coord(0), coord(97), coord(0), coord(98)), ascii,
                "charwise ascii split");
    }

    @Test
    void charWiseHangul() {
        // Rust charwise_hangul: U+D55C = 54620, c0 = 4, c1 = 9932.
        List<Coord> hangul = generate(CharWise.INSTANCE, "\uD55C");
        assertEquals(List.of(coord(4), coord(9932)), hangul, "charwise hangul pair");

        // The C++ case checks the same shape for U+AC00.
        assertEquals(2, generate(CharWise.INSTANCE, "\uAC00").size(), "charwise hangul pair size");
    }

    @Test
    void charWiseRejectsUnpairedSurrogate() {
        assertTrue(CharWise.INSTANCE.generate("\uD800").isEmpty(),
                "unpaired surrogate is not a scalar value");
    }

    @Test
    void charWiseIsInjective() {
        // The same shared-prefix set: "ab" and "ac" share the whole first
        // scalar pair, "ba" collides under a scalar-set mapping, and "abc"
        // under a pair-truncating one.
        String[] keys = {"ab", "ac", "ba", "abc"};
        Set<List<Coord>> paths = new HashSet<>();
        for (String key : keys) {
            paths.add(generate(CharWise.INSTANCE, key));
        }
        assertEquals(keys.length, paths.size(), "different keys must produce different paths");
        assertEquals(generate(CharWise.INSTANCE, "ab").subList(0, 2),
                generate(CharWise.INSTANCE, "ac").subList(0, 2),
                "the shared scalar prefix maps to the same pair");
    }

    @Test
    void prefix() {
        List<Coord> coords = generate(new Prefix(4), "hi");
        assertEquals(4, coords.size(), "prefix length 4");
        assertEquals('h', coords.get(0).index(), "prefix first index");
        assertEquals('i', coords.get(1).index(), "prefix second index");
        assertEquals(0, coords.get(2).index(), "prefix zero pad");
        assertEquals(0, coords.get(3).index(), "prefix zero pad");
        assertFalse(new Prefix(4).isInjective(), "prefix lossy");
        assertEquals(OptionalInt.of(4), new Prefix(4).fixedDepth(), "prefix fixed depth");
        assertEquals(4, new Prefix(4).length(), "prefix length accessor");
        assertEquals("prefix", new Prefix(4).name(), "prefix name");

        // Rust prefix_basic: Prefix<3> takes the first three bytes.
        assertEquals(List.of(coord('a'), coord('b'), coord('c')),
                generate(new Prefix(3), "abcde"), "prefix takes the leading bytes");
    }

    @Test
    void byteFold() {
        List<Coord> coords = generate(new ByteFold(2), "ab");
        assertEquals(2, coords.size(), "bytefold length");
        assertEquals('a', coords.get(0).index(), "bytefold first accumulator");
        assertEquals('b', coords.get(1).index(), "bytefold second accumulator");
        assertEquals(new ByteFold(2).generate("ab"), new ByteFold(2).generate("ab"),
                "bytefold deterministic");
        assertFalse(new ByteFold(2).isInjective(), "bytefold lossy");
        assertEquals(OptionalInt.of(2), new ByteFold(2).fixedDepth(), "bytefold fixed depth");
        assertEquals(2, new ByteFold(2).length(), "bytefold length accessor");
        assertEquals("byte-fold", new ByteFold(2).name(), "bytefold name");
    }

    @Test
    void stringToCoordPath() {
        Optional<List<Coord>> fromFunction = CoordGen.stringToCoordPath("key");
        Optional<List<Coord>> fromStrategy = ByteWise.INSTANCE.generate("key");
        assertTrue(fromFunction.isPresent(), "string to path present");
        assertEquals(fromStrategy, fromFunction, "string_to_coord_path matches bytewise");
        assertTrue(CoordGen.stringToCoordPath("").isEmpty(), "empty rejected");
    }

    @Test
    void defaultDynamicIsByteWise() {
        assertSame(ByteWise.INSTANCE, CoordGen.DEFAULT_DYNAMIC, "default dynamic strategy");
    }

    @Test
    void strategyEdgeCases() {
        // Empty keys are rejected by every strategy.
        assertTrue(new Prefix(2).generate("").isEmpty(), "prefix empty rejected");
        assertTrue(new ByteFold(2).generate("").isEmpty(), "bytefold empty rejected");

        // ByteFold: XOR is commutative within each accumulator, so swapping
        // even-position bytes between two strings produces a collision.
        String keyA = "a\u0000c";
        String keyB = "c\u0000a";
        Optional<List<Coord>> pathA = new ByteFold(2).generate(keyA);
        Optional<List<Coord>> pathB = new ByteFold(2).generate(keyB);
        assertTrue(pathA.isPresent() && pathB.isPresent(), "bytefold collision paths present");
        assertEquals(pathA, pathB, "bytefold commutative xor collision");
        List<Coord> collision = pathA.orElseThrow();
        assertEquals(2, collision.get(0).index(), "bytefold collision first accumulator");
        assertEquals(0, collision.get(1).index(), "bytefold collision second accumulator");

        // Rust bytefold_basic: "abcd" folds to 'a'^'c' and 'b'^'d'.
        List<Coord> folded = generate(new ByteFold(2), "abcd");
        assertEquals(2, folded.size(), "bytefold path length");
        assertEquals('a' ^ 'c', folded.get(0).index(), "bytefold accumulator values");
        assertEquals('b' ^ 'd', folded.get(1).index(), "bytefold accumulator values");

        // Prefix truncation: keys sharing the first N bytes collide.
        assertEquals(new Prefix(2).generate("ab"), new Prefix(2).generate("abcdef"),
                "prefix truncation collision");

        // Prefix<2> matches the first two ByteWise coordinates.
        List<Coord> prefixed = generate(new Prefix(2), "abcd");
        List<Coord> bytewise = generate(ByteWise.INSTANCE, "abcd");
        assertEquals(bytewise.get(0), prefixed.get(0), "prefix2 matches bytewise first");
        assertEquals(bytewise.get(1), prefixed.get(1), "prefix2 matches bytewise second");
    }

    @Test
    void zeroLengthStrategiesRejected() {
        assertThrows(IllegalArgumentException.class, () -> new Prefix(0),
                "prefix depth zero rejected");
        assertThrows(IllegalArgumentException.class, () -> new ByteFold(0),
                "bytefold depth zero rejected");
    }

    private static Coord coord(int index) {
        return Coord.fromIndex(index).orElseThrow();
    }

    private static List<Coord> generate(CoordGen strategy, String key) {
        return strategy.generate(key).orElseThrow();
    }

    private static void assertValue(byte[] expected, Optional<byte[]> actual, String message) {
        assertTrue(actual.isPresent(), message + ": value present");
        assertArrayEquals(expected, actual.orElseThrow(), message);
    }
}
