package org.ssccs.syntagma.core;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.Test;

/**
 * Translation of the C++ test suite {@code sw/cpp/tagma_core/tests/test_coord.cpp}:
 * closed-form composition, decomposition, validity, axes, code point
 * conversion, Hamming distance, serialization, and ordering.
 */
class CoordTest {

    private static Coord axes(int initial, int medial, int final_) {
        return Coord.fromAxes(initial, medial, final_).orElseThrow();
    }

    private static Coord index(int index) {
        return Coord.fromIndex(index).orElseThrow();
    }

    @Test
    void closedFormArithmetic() {
        assertEquals(3235, axes(5, 10, 15).index());
        assertEquals(0xB8A3, axes(5, 10, 15).codePoint());
        assertEquals('가', index(0).toChar());
        assertTrue(axes(1, 0, 0).compareTo(axes(2, 0, 0)) < 0);
        assertEquals(new Coord.Axes(18, 20, 27), axes(18, 20, 27).hammingDistance(axes(0, 0, 0)));
    }

    @Test
    void latticeEdgesAndMidpointsRoundTrip() {
        int[][] lattice = {{0, 0, 0}, {18, 20, 27}, {5, 10, 15}, {0, 20, 27}, {18, 0, 27}, {18, 20, 0}};
        for (int[] axes : lattice) {
            Coord coord = axes(axes[0], axes[1], axes[2]);
            assertEquals(new Coord.Axes(axes[0], axes[1], axes[2]), coord.axes(), "axes round trip");
            assertEquals(axes[0] * 588 + axes[1] * 28 + axes[2], coord.index(), "index composition");
            assertEquals(Coord.BASE + axes[0] * 588 + axes[1] * 28 + axes[2], coord.codePoint(),
                    "code point composition");
            assertTrue(coord.valid(), "constructed coord valid");
        }
    }

    @Test
    void invalidAxesRejected() {
        assertTrue(Coord.fromAxes(-1, 0, 0).isEmpty(), "negative initial rejected");
        assertTrue(Coord.fromAxes(19, 0, 0).isEmpty(), "initial overflow rejected");
        assertTrue(Coord.fromAxes(0, 21, 0).isEmpty(), "medial overflow rejected");
        assertTrue(Coord.fromAxes(0, 0, 28).isEmpty(), "final overflow rejected");
    }

    @Test
    void indexBounds() {
        assertEquals(index(0), index(0));
        assertTrue(Coord.fromIndex(0).isPresent(), "index 0 valid");
        assertTrue(Coord.fromIndex(11171).isPresent(), "index 11171 valid");
        assertTrue(Coord.fromIndex(11172).isEmpty(), "index 11172 rejected");
        assertEquals('가', index(0).toChar(), "block base char");
        assertEquals('힣', index(11171).toChar(), "block end char");
        assertEquals(0xAC00, index(0).codePoint(), "block base code point");
        assertEquals(0xD7A3, index(11171).codePoint(), "block end code point");
    }

    @Test
    void codePointBoundsIncludingFillers() {
        assertTrue(Coord.fromCodePoint(0xAC00).isPresent(), "code point base valid");
        assertTrue(Coord.fromCodePoint(0xD7A3).isPresent(), "code point end valid");
        assertTrue(Coord.fromCodePoint(0xABFF).isEmpty(), "below base rejected");
        // Every filler position U+D7A4..U+D7AF lacks structural validity.
        for (int cp = 0xD7A4; cp <= 0xD7AF; cp++) {
            assertTrue(Coord.fromCodePoint(cp).isEmpty(), "filler rejected: U+" + Integer.toHexString(cp));
        }
        assertTrue(Coord.fromCodePoint(0xD7B0).isEmpty(), "above block rejected");
        assertTrue(Coord.fromChar('\uD7B0').isEmpty(), "out of block char rejected");
    }

    @Test
    void hammingDistanceIsPerAxisAbsoluteDifference() {
        Coord a = axes(0, 0, 0);
        Coord b = axes(1, 0, 0);
        Coord c = axes(0, 1, 0);
        Coord d = axes(0, 0, 1);
        Coord e = axes(3, 5, 7);
        assertEquals(new Coord.Axes(0, 0, 0), a.hammingDistance(a), "self hamming zero");
        assertEquals(new Coord.Axes(1, 0, 0), a.hammingDistance(b), "initial axis difference");
        assertEquals(new Coord.Axes(0, 1, 0), a.hammingDistance(c), "medial axis difference");
        assertEquals(new Coord.Axes(0, 0, 1), a.hammingDistance(d), "final axis difference");
        assertEquals(new Coord.Axes(3, 5, 7), a.hammingDistance(e), "three-axis difference");
        assertEquals(new Coord.Axes(1, 1, 0), b.hammingDistance(c), "two-axis difference");
    }

    @Test
    void constants() {
        assertEquals(65536 - Coord.N_VALID, Coord.INVALID_MARGIN, "invalidity margin constant");
        assertEquals(65536, Coord.TOTAL, "total states constant");
    }

    @Test
    void hangulStringDisplay() {
        assertEquals("가", index(0).toHangulString(), "hangul string base");
        assertEquals("힣", index(11171).toHangulString(), "hangul string end");
    }

    @Test
    void byteSerialization() {
        // Round trip over the same sample set as the Rust serialization_roundtrip.
        for (int raw : new int[] {0, 1, 256, 11171, 5555}) {
            Coord coord = index(raw);
            assertEquals(coord, Coord.fromLeBytes(coord.toLeBytes()).orElseThrow(), "le roundtrip");
            assertEquals(coord, Coord.fromBeBytes(coord.toBeBytes()).orElseThrow(), "be roundtrip");
        }
        byte[] leLast = {(byte) 0xA3, 0x2B}; // 11171 = 0x2BA3, LE
        byte[] beLast = {0x2B, (byte) 0xA3}; // 11171 BE
        assertArrayEquals(leLast, index(11171).toLeBytes());
        assertArrayEquals(beLast, index(11171).toBeBytes());
        byte[] leInvalid = {(byte) 0xA4, 0x2B}; // 11172 LE
        byte[] beInvalid = {0x2B, (byte) 0xA4}; // 11172 BE
        assertTrue(Coord.fromLeBytes(leInvalid).isEmpty(), "invalid le bytes rejected");
        assertTrue(Coord.fromBeBytes(beInvalid).isEmpty(), "invalid be bytes rejected");
    }

    @Test
    void orderingByIndex() {
        Coord low = index(0);
        Coord high = index(11171);
        assertTrue(low.compareTo(high) < 0);
        assertTrue(low.compareTo(low) == 0);
        assertTrue(high.compareTo(low) > 0);
        assertFalse(low.equals(high), "inequality");
        assertEquals(low, low, "equality");
    }

    @Test
    void exhaustiveLatticeScan() {
        for (int raw = 0; raw < Coord.N_VALID; raw++) {
            Coord coord = index(raw);
            assertTrue(coord.valid(), "all " + Coord.N_VALID + " indices valid");
            Coord.Axes axes = coord.axes();
            assertEquals(coord, Coord.fromAxes(axes.initial(), axes.medial(), axes.final_()).orElseThrow(),
                    "axes roundtrip");
            assertEquals(coord, Coord.fromCodePoint(coord.codePoint()).orElseThrow(), "code point roundtrip");
            assertEquals(coord, Coord.fromChar(coord.toChar()).orElseThrow(), "char roundtrip");
        }
        assertTrue(Coord.fromIndex(Coord.N_VALID).isEmpty(), "first invalid index rejected");
    }
}
