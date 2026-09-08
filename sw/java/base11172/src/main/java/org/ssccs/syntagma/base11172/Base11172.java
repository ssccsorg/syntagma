package org.ssccs.syntagma.base11172;

import java.util.Optional;
import java.util.OptionalInt;
import org.ssccs.syntagma.core.Coord;

/**
 * Tagma native serialization format.
 *
 * <p>Encodes arbitrary byte sequences into self-validating compositional
 * character strings using the Tagma coordinate space as its alphabet.
 * Properties: self-validating (invalid characters are immediately
 * detectable), no special characters (URL-safe, no escaping needed), and
 * deterministic (encoding is a pure function of the input).
 *
 * <p>The C++ reference operates on {@code std::u32string} and the Rust
 * reference on UTF-8 {@code String}; Java uses {@link String}, whose UTF-16
 * code units carry the BMP composition characters directly.
 *
 * <p>Port of the C++ {@code base11172} module in {@code sw/cpp/base11172};
 * the underlying behavior mirrors the Rust {@code base11172} crate in
 * {@code sw/rust/base11172}.
 */
public final class Base11172 {

    /** The number of distinct characters used by this encoding (11,172). */
    public static final int N_CHARS = Coord.N_VALID;

    private Base11172() {
    }

    /**
     * Encodes a 16-bit value into two compositional characters (base-11172).
     * Each character carries log2(11172) about 13.45 bits, so a pair covers a
     * full 16-bit value.
     */
    public static char[] encodeU16(int value) {
        int hi = value / N_CHARS;
        int lo = value % N_CHARS;
        return new char[] {coordChar(hi), coordChar(lo)};
    }

    /**
     * Encodes bytes into a compositional string, 2 bytes per character pair.
     * An odd trailing byte is encoded as a pair with an implicit zero high
     * byte.
     */
    public static String encodeBytes(byte[] data) {
        StringBuilder out = new StringBuilder(data.length);
        for (int i = 0; i < data.length; i += 2) {
            int value = (i + 1 < data.length)
                    ? (data[i] & 0xFF) | ((data[i + 1] & 0xFF) << 8)
                    : (data[i] & 0xFF);
            out.append(encodeU16(value));
        }
        return out.toString();
    }

    /**
     * Decodes a pair of compositional characters back to a 16-bit value.
     *
     * @return empty when either character is outside the valid compositional
     *         block (U+AC00..U+D7AF)
     */
    public static OptionalInt decodePair(char c0, char c1) {
        Optional<Coord> coord0 = Coord.fromChar(c0);
        Optional<Coord> coord1 = Coord.fromChar(c1);
        if (coord0.isEmpty() || coord1.isEmpty()) {
            return OptionalInt.empty();
        }
        int value = coord0.get().index() * N_CHARS + coord1.get().index();
        return OptionalInt.of(value & 0xFFFF);
    }

    /**
     * Decodes a string back to bytes, 2 characters per 16-bit pair.
     *
     * @return empty when the string has an odd character count or any invalid
     *         character
     */
    public static Optional<byte[]> decodeBytes(String text) {
        if ((text.length() & 1) != 0) {
            return Optional.empty();
        }
        byte[] out = new byte[text.length()];
        for (int i = 0; i < text.length(); i += 2) {
            OptionalInt value = decodePair(text.charAt(i), text.charAt(i + 1));
            if (value.isEmpty()) {
                return Optional.empty();
            }
            int v = value.getAsInt();
            out[i] = (byte) (v & 0xFF);
            out[i + 1] = (byte) (v >> 8);
        }
        return Optional.of(out);
    }

    private static char coordChar(int index) {
        // hi and lo are always below N_CHARS for a 16-bit input, so the
        // coordinate is always valid; fall back to the block base otherwise.
        return Coord.fromIndex(index).orElse(Coord.ZERO).toChar();
    }
}
