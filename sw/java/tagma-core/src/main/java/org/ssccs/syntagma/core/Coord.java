package org.ssccs.syntagma.core;

import java.util.Optional;

/**
 * A 16-bit value guaranteed to be a valid compositional character coordinate.
 *
 * <p>{@code Coord} wraps an index in {@code 0..11171}, the offset inside the
 * Unicode closed-form composition block U+AC00..U+D7AF. Every valid value is
 * simultaneously a Unicode address, a 3-axis coordinate (initial, medial,
 * final), and a compositional character. Composition and decomposition are
 * closed-form arithmetic with no hash function:
 *
 * <pre>{@code
 * C(i, m, f) = 0xAC00 + 588 * i + 28 * m + f
 *              for 0 <= i < 19, 0 <= m < 21, 0 <= f < 28
 * }</pre>
 *
 * <p>The lattice holds {@link #N_VALID} (11,172) valid coordinates. The
 * remaining 54,364 of the 65,536 representable 16-bit states are structurally
 * invalid and detectable in constant time.
 *
 * <p>Port of the C++ {@code tagma::Coord} in {@code sw/cpp/tagma_core}; the
 * underlying behavior mirrors the Rust {@code Coord} in {@code sw/rust/core}.
 */
public record Coord(int index) implements Comparable<Coord> {

    /** Base code point (U+AC00). */
    public static final int BASE = 0xAC00;

    /**
     * Last code point of the Unicode compositional characters block (U+D7AF).
     * U+D7A4..U+D7AF are filler positions; the last valid character is at
     * U+D7A3 (offset 11171).
     */
    public static final int LAST = 0xD7AF;

    /** Number of initial (choseong) axes. */
    public static final int INITIAL_MAX = 19;

    /** Number of medial (jungseong) axes. */
    public static final int MEDIAL_MAX = 21;

    /** Number of final (jongseong) axes. */
    public static final int FINAL_MAX = 28;

    /** Number of valid character blocks (19 x 21 x 28). */
    public static final int N_VALID = 11172;

    /** Number of representable 16-bit states. */
    public static final int TOTAL = 65536;

    /** Number of structurally invalid states ({@link #TOTAL} - {@link #N_VALID}). */
    public static final int INVALID_MARGIN = TOTAL - N_VALID;

    private static final int STRIDE_MED = FINAL_MAX; // 28
    private static final int STRIDE_INIT = MEDIAL_MAX * FINAL_MAX; // 588

    /** The zero coordinate (index 0), the block base character '가'. */
    public static final Coord ZERO = raw(0);

    /**
     * Compact canonical constructor. Every publicly constructible {@code Coord}
     * satisfies {@code 0 <= index < N_VALID}; the validating factories below
     * never invoke it with an out-of-range index.
     *
     * @throws IllegalArgumentException if {@code index} is outside 0..11171
     */
    public Coord {
        if (index < 0 || index >= N_VALID) {
            throw new IllegalArgumentException("Coord index out of range: " + index);
        }
    }

    // ------------------------------------------------------------------
    // Construction
    // ------------------------------------------------------------------

    /**
     * Creates a {@code Coord} from the raw 0-based index.
     *
     * @return empty when {@code index} is outside [0, 11171]
     */
    public static Optional<Coord> fromIndex(int index) {
        return index >= 0 && index < N_VALID
                ? Optional.of(new Coord(index))
                : Optional.empty();
    }

    /**
     * Creates a {@code Coord} from its three structural axes.
     *
     * @return empty when any axis is out of bounds. Valid ranges:
     *         initial 0..18, medial 0..20, final 0..27
     */
    public static Optional<Coord> fromAxes(int initial, int medial, int final_) {
        if (initial < 0 || initial >= INITIAL_MAX
                || medial < 0 || medial >= MEDIAL_MAX
                || final_ < 0 || final_ >= FINAL_MAX) {
            return Optional.empty();
        }
        return Optional.of(new Coord(initial * STRIDE_INIT + medial * STRIDE_MED + final_));
    }

    /**
     * Creates a {@code Coord} from a Unicode code point U+AC00..U+D7AF.
     *
     * @return empty for code points outside the composition block and for the
     *         filler positions U+D7A4..U+D7AF, which lack structural validity
     */
    public static Optional<Coord> fromCodePoint(int codePoint) {
        if (codePoint < BASE || codePoint > LAST) {
            return Optional.empty();
        }
        int offset = codePoint - BASE;
        return offset < N_VALID ? Optional.of(new Coord(offset)) : Optional.empty();
    }

    /**
     * Creates a {@code Coord} from a character.
     *
     * @return empty when {@code ch} is not a valid compositional character
     */
    public static Optional<Coord> fromChar(char ch) {
        return fromCodePoint(ch & 0xFFFF);
    }

    // ------------------------------------------------------------------
    // Serialization
    // ------------------------------------------------------------------

    /** Little-endian bytes of the raw index. */
    public byte[] toLeBytes() {
        return new byte[] {(byte) (index() & 0xFF), (byte) (index() >> 8)};
    }

    /** Big-endian bytes of the raw index. */
    public byte[] toBeBytes() {
        return new byte[] {(byte) (index() >> 8), (byte) (index() & 0xFF)};
    }

    /**
     * Creates a {@code Coord} from little-endian bytes of the raw index.
     *
     * @return empty when the decoded index is invalid
     */
    public static Optional<Coord> fromLeBytes(byte[] bytes) {
        requireBytes(bytes);
        return fromIndex((bytes[0] & 0xFF) | ((bytes[1] & 0xFF) << 8));
    }

    /**
     * Creates a {@code Coord} from big-endian bytes of the raw index.
     *
     * @return empty when the decoded index is invalid
     */
    public static Optional<Coord> fromBeBytes(byte[] bytes) {
        requireBytes(bytes);
        return fromIndex(((bytes[0] & 0xFF) << 8) | (bytes[1] & 0xFF));
    }

    private static void requireBytes(byte[] bytes) {
        if (bytes == null || bytes.length != 2) {
            throw new IllegalArgumentException("Coord byte serialization requires exactly 2 bytes");
        }
    }

    // ------------------------------------------------------------------
    // Decomposition and distance
    // ------------------------------------------------------------------

    /** Structural validity. Every constructed {@code Coord} is valid. */
    public boolean valid() {
        return index() < N_VALID;
    }

    /** Unicode code point U+AC00..U+D7AF. */
    public int codePoint() {
        return BASE + index();
    }

    /** Compositional character display of the code point. */
    public char toChar() {
        return (char) codePoint();
    }

    /** The compositional character as a one-character string. */
    public String toHangulString() {
        return String.valueOf(toChar());
    }

    /**
     * Decomposes this coordinate into its three structural axes.
     *
     * <p>The third component is named {@code final_} because {@code final} is
     * a reserved word in Java; it mirrors the final (jongseong) axis.
     */
    public Axes axes() {
        int initial = index() / STRIDE_INIT;
        int remainder = index() % STRIDE_INIT;
        int medial = remainder / STRIDE_MED;
        int final_ = remainder % STRIDE_MED;
        return new Axes(initial, medial, final_);
    }

    /** Field-wise Hamming distance: per-axis absolute differences. */
    public Axes hammingDistance(Coord other) {
        Axes a = this.axes();
        Axes b = other.axes();
        return new Axes(absDiff(a.initial(), b.initial()),
                absDiff(a.medial(), b.medial()),
                absDiff(a.final_(), b.final_()));
    }

    private static int absDiff(int a, int b) {
        return a > b ? a - b : b - a;
    }

    @Override
    public int compareTo(Coord other) {
        return Integer.compare(index(), other.index());
    }

    /** Displays the compositional character, mirroring the C++ stream output. */
    @Override
    public String toString() {
        return toHangulString();
    }

    /**
     * Package-private fast path for collection internals that already hold a
     * validated index. The index must satisfy {@code 0 <= index < N_VALID}.
     */
    static Coord raw(int index) {
        return new Coord(index);
    }

    /**
     * The three structural axes of a {@link Coord}, returned by
     * {@link Coord#axes()} and {@link Coord#hammingDistance(Coord)}. The third
     * component is named {@code final_} because {@code final} is a reserved
     * word in Java.
     */
    public record Axes(int initial, int medial, int final_) {}
}
