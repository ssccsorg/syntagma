package org.ssccs.syntagma.base11172;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.Arrays;
import java.util.Optional;
import java.util.OptionalInt;
import org.junit.jupiter.api.Test;

/**
 * Translation of the C++ test suite {@code sw/cpp/base11172/tests/test_base11172.cpp}:
 * u16 round trip, byte round trip, binary round trip, odd-byte padding, and
 * invalid input rejection.
 */
class Base11172Test {

    @Test
    void roundTripU16() {
        int[] values = {0, 1, 11171, 12345, 32768, 65535};
        for (int value : values) {
            char[] pair = Base11172.encodeU16(value);
            assertEquals(OptionalInt.of(value), Base11172.decodePair(pair[0], pair[1]),
                    "u16 round trip for " + value);
        }
    }

    @Test
    void roundTripBytes() {
        // 17 bytes (odd): the decoded vector carries one zero padding byte,
        // so the reference compares only the leading 17 bytes.
        byte[] data = "Hello, Base11172!".getBytes(java.nio.charset.StandardCharsets.UTF_8);
        assertEquals(17, data.length);
        String encoded = Base11172.encodeBytes(data);
        byte[] decoded = Base11172.decodeBytes(encoded).orElseThrow();
        assertTrue(decoded.length >= data.length, "bytes decode");
        assertArrayEquals(data, Arrays.copyOf(decoded, data.length), "bytes round trip");
    }

    @Test
    void binaryRoundTrip() {
        byte[] data = new byte[256];
        for (int i = 0; i < 256; i++) {
            data[i] = (byte) i;
        }
        String encoded = Base11172.encodeBytes(data);
        assertArrayEquals(data, Base11172.decodeBytes(encoded).orElseThrow(), "binary round trip");
    }

    @Test
    void oddBytePadding() {
        byte[] single = {0x41};
        String encoded = Base11172.encodeBytes(single);
        byte[] decoded = Base11172.decodeBytes(encoded).orElseThrow();
        assertEquals(2, decoded.length);
        assertEquals(0x41, decoded[0] & 0xFF, "odd byte padded with zero high byte");
        assertEquals(0x00, decoded[1] & 0xFF);
    }

    @Test
    void invalidInput() {
        assertTrue(Base11172.decodePair('\0', '가').isEmpty(), "null char rejected");
        assertTrue(Base11172.decodePair('가', '\0').isEmpty(), "null second char rejected");
        assertTrue(Base11172.decodePair('\uD7B0', '가').isEmpty(), "char beyond block rejected");
        assertTrue(Base11172.decodePair('가', '\uD7B0').isEmpty(), "second char beyond block rejected");
        assertTrue(Base11172.decodeBytes("가").isEmpty(), "odd count rejected");
        assertTrue(Base11172.decodeBytes("가\uD7B0").isEmpty(), "invalid char rejected");
    }
}
