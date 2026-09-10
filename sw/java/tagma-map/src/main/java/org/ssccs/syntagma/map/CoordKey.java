package org.ssccs.syntagma.map;

import java.nio.charset.StandardCharsets;
import java.util.Arrays;
import java.util.Objects;

import org.ssccs.syntagma.core.Coord;
import org.ssccs.syntagma.core.CoordPath;

/**
 * A fixed-size byte-array key that maps injectively to a {@link CoordPath} of
 * the same length.
 *
 * <p>Unlike {@link Prefix} (which truncates arbitrary strings) the key carries
 * the exact length as immutable instance state, so a key can never hold a
 * different number of bytes than the store that owns it expects.
 *
 * <p>The byte-space domain of one key character is {@link #BYTE_DOMAIN} (256):
 * every key character addresses one byte. A {@link CoordPath} index at or above
 * that domain cannot be represented as a byte key, so
 * {@link #fromCoordPath(CoordPath)} rejects it instead of truncating; silently
 * folding such an index would collide with a distinct entry, for example index
 * 0 and index 256.
 *
 * <p>Port of the C++ {@code tagma_map::CoordKey<N>} in
 * {@code sw/cpp/tagma_map/include/tagma_map/coord_key.h}; the underlying
 * behavior mirrors the Rust {@code CoordKey<N>} in
 * {@code sw/rust/map/src/coord_gen.rs}.
 */
public final class CoordKey {

    /**
     * The byte-space domain of one key character, 256. Each key character
     * addresses one byte of the {@code [0, 256)} domain. Mirrors the C++
     * {@code CoordKey<N>::kByteDomain} and the Rust
     * {@code CoordKey::BYTE_DOMAIN}.
     */
    public static final int BYTE_DOMAIN = 256;

    private final byte[] bytes;

    /**
     * Creates a key from a byte array of any length. The key takes a defensive
     * copy, so later changes to {@code bytes} do not affect the key.
     *
     * @throws NullPointerException when {@code bytes} is null
     */
    public CoordKey(byte[] bytes) {
        Objects.requireNonNull(bytes, "bytes");
        this.bytes = bytes.clone();
    }

    /**
     * Creates a key from one byte per character index. Mirrors the C++ and Rust
     * byte-array construction with the runtime index validation the const
     * generic parameter cannot express in Java.
     *
     * @return a key holding {@code indices.length} bytes
     * @throws IllegalArgumentException when an index is outside the byte-space
     *         domain {@code [0, BYTE_DOMAIN)}
     * @throws NullPointerException when {@code indices} is null
     */
    public static CoordKey fromIndices(int[] indices) {
        Objects.requireNonNull(indices, "indices");
        byte[] bytes = new byte[indices.length];
        for (int i = 0; i < indices.length; i++) {
            int index = indices[i];
            if (index < 0 || index >= BYTE_DOMAIN) {
                throw new IllegalArgumentException(
                        "CoordKey::fromIndices: character " + i + " index " + index
                                + " is outside the byte-space domain [0, " + BYTE_DOMAIN + ")");
            }
            bytes[i] = (byte) index;
        }
        return new CoordKey(bytes);
    }

    /**
     * Creates a key from a string of exactly {@code length} bytes. The string
     * is encoded as UTF-8, mirroring the byte string of the references, whose
     * length is fixed at compile time while Java carries it as a parameter.
     *
     * @throws IllegalArgumentException when the UTF-8 byte length of
     *         {@code key} differs from {@code length}, mirroring the always-on
     *         panic of the Rust {@code From<&str>} implementation
     * @throws NullPointerException when {@code key} is null
     */
    public static CoordKey fromString(String key, int length) {
        Objects.requireNonNull(key, "key");
        byte[] bytes = key.getBytes(StandardCharsets.UTF_8);
        if (bytes.length != length) {
            throw new IllegalArgumentException(
                    "CoordKey::fromString: expected a string of exactly " + length
                            + " bytes, got " + bytes.length);
        }
        return new CoordKey(bytes);
    }

    /**
     * Creates a key from a path: each character index becomes one key byte.
     *
     * @throws IllegalArgumentException when a character index is at or above
     *         {@link #BYTE_DOMAIN}, mirroring the always-on panic of the Rust
     *         {@code from_coord_path} and the {@code std::invalid_argument} of
     *         the C++ port
     * @throws NullPointerException when {@code path} is null
     */
    public static CoordKey fromCoordPath(CoordPath path) {
        Objects.requireNonNull(path, "path");
        Coord[] coords = path.coords();
        byte[] bytes = new byte[coords.length];
        for (int i = 0; i < coords.length; i++) {
            int index = coords[i].index();
            if (index >= BYTE_DOMAIN) {
                throw new IllegalArgumentException(
                        "CoordKey::fromCoordPath: character " + i + " index " + index
                                + " exceeds the byte-space domain [0, " + BYTE_DOMAIN + ")");
            }
            bytes[i] = (byte) index;
        }
        return new CoordKey(bytes);
    }

    /** The number of bytes in the key, nothing else. */
    public int length() {
        return bytes.length;
    }

    /** Whether the key holds no bytes, the degenerate length-zero case. */
    public boolean isEmpty() {
        return bytes.length == 0;
    }

    /** The underlying bytes as a defensive copy. */
    public byte[] bytes() {
        return bytes.clone();
    }

    /**
     * The injective {@link CoordPath} mapping: each byte maps to one
     * {@code Coord}, so a key of length {@code N} produces a unique path.
     */
    public CoordPath toCoordPath() {
        Coord[] coords = new Coord[bytes.length];
        for (int i = 0; i < bytes.length; i++) {
            coords[i] = Coord.fromIndex(bytes[i] & 0xFF).orElseThrow();
        }
        return CoordPath.fromArray(coords);
    }

    @Override
    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        if (!(other instanceof CoordKey key)) {
            return false;
        }
        return Arrays.equals(bytes, key.bytes);
    }

    @Override
    public int hashCode() {
        return Arrays.hashCode(bytes);
    }
}
