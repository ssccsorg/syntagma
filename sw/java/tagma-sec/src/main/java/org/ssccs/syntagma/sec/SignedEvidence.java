package org.ssccs.syntagma.sec;

import java.util.Arrays;
import java.util.Objects;

/**
 * Signed evidence for non-repudiation of origin. The tag is the keyed
 * commitment to the evidence; verification recomputes it and compares.
 *
 * <p>Port of the C++ {@code tagma_sec::SignedEvidence} in
 * {@code sw/cpp/tagma_sec/include/tagma_sec/types.h}; the underlying behavior
 * mirrors the Rust {@code SignedEvidence} in {@code sw/rust/sec/src/types.rs}.
 * Both byte sequences are copied on the way in and cloned on the way out.
 *
 * @param evidence the signed message
 * @param tag      the 32-byte keyed commitment to {@code evidence}
 */
public record SignedEvidence(byte[] evidence, byte[] tag) {

    /**
     * Compact canonical constructor: validates the tag length and stores
     * defensive copies.
     *
     * @throws IllegalArgumentException when {@code tag} is not 32 bytes long
     */
    public SignedEvidence {
        Objects.requireNonNull(evidence, "evidence");
        Objects.requireNonNull(tag, "tag");
        if (tag.length != Hashes.TAG_BYTES) {
            throw new IllegalArgumentException(
                    "SignedEvidence tag must be " + Hashes.TAG_BYTES + " bytes, got " + tag.length);
        }
        evidence = evidence.clone();
        tag = tag.clone();
    }

    /** The evidence as a defensive copy. */
    @Override
    public byte[] evidence() {
        return evidence.clone();
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
        if (!(other instanceof SignedEvidence signed)) {
            return false;
        }
        return Arrays.equals(evidence, signed.evidence) && Arrays.equals(tag, signed.tag);
    }

    @Override
    public int hashCode() {
        return 31 * Arrays.hashCode(evidence) + Arrays.hashCode(tag);
    }
}
