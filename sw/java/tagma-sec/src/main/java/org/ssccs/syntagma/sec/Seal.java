package org.ssccs.syntagma.sec;

import java.util.Arrays;
import java.util.Objects;

/**
 * Integrity binding over a record, its path, a principal, and an epoch.
 *
 * <p>The tag is a keyed commitment of 32 bytes. It does not hide the record;
 * it makes modification evident. {@code equals} and {@code hashCode} compare
 * and fold the tag contents, mirroring the element-wise comparison of the C++
 * {@code std::array} member.
 *
 * <p>Port of the C++ {@code tagma_sec::Seal} in
 * {@code sw/cpp/tagma_sec/include/tagma_sec/types.h}; the underlying behavior
 * mirrors the Rust {@code Seal} in {@code sw/rust/sec/src/types.rs}.
 *
 * @param tag the 32-byte keyed commitment
 */
public record Seal(byte[] tag) {

    /**
     * Compact canonical constructor: validates the tag length and stores a
     * defensive copy.
     *
     * @throws IllegalArgumentException when {@code tag} is not 32 bytes long
     */
    public Seal {
        Objects.requireNonNull(tag, "tag");
        if (tag.length != Hashes.TAG_BYTES) {
            throw new IllegalArgumentException(
                    "Seal tag must be " + Hashes.TAG_BYTES + " bytes, got " + tag.length);
        }
        tag = tag.clone();
    }

    /** The 32-byte tag as a defensive copy. */
    @Override
    public byte[] tag() {
        return tag.clone();
    }

    @Override
    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        if (!(other instanceof Seal seal)) {
            return false;
        }
        return Arrays.equals(tag, seal.tag);
    }

    @Override
    public int hashCode() {
        return Arrays.hashCode(tag);
    }
}
