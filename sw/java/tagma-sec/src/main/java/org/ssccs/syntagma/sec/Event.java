package org.ssccs.syntagma.sec;

import java.util.Arrays;
import java.util.Objects;
import java.util.OptionalLong;

/**
 * An entry in the append-only audit log.
 *
 * <p>{@code prev} chains the entry to its predecessor: it is empty exactly for
 * genesis, the entry with id 0. {@code payloadHash} is the SHA-256 commitment
 * to the payload in place of the Rust blake3 commitment.
 *
 * <p>Port of the C++ {@code tagma_sec::Event} in
 * {@code sw/cpp/tagma_sec/include/tagma_sec/types.h}; the underlying behavior
 * mirrors the Rust {@code Event} in {@code sw/rust/sec/src/types.rs}. The C++
 * {@code std::optional<uint64_t> prev} maps to {@link OptionalLong}, and the
 * C++ {@code std::array<uint8_t, 32> payload_hash} maps to a 32-byte array
 * copied on the way in and cloned on the way out.
 *
 * @param id          entry identity, its index in the log
 * @param prev        id of the preceding entry, empty for genesis
 * @param payloadHash 32-byte commitment to the payload
 * @param epoch       epoch recorded at append time
 */
public record Event(long id, OptionalLong prev, byte[] payloadHash, long epoch) {

    /**
     * Compact canonical constructor: validates the commitment length and
     * stores a defensive copy.
     *
     * @throws IllegalArgumentException when {@code payloadHash} is not 32 bytes long
     */
    public Event {
        Objects.requireNonNull(prev, "prev");
        Objects.requireNonNull(payloadHash, "payloadHash");
        if (payloadHash.length != Hashes.TAG_BYTES) {
            throw new IllegalArgumentException(
                    "Event payload hash must be " + Hashes.TAG_BYTES + " bytes, got " + payloadHash.length);
        }
        payloadHash = payloadHash.clone();
    }

    /** The 32-byte payload commitment as a defensive copy. */
    @Override
    public byte[] payloadHash() {
        return payloadHash.clone();
    }

    @Override
    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        if (!(other instanceof Event event)) {
            return false;
        }
        return id == event.id
                && epoch == event.epoch
                && prev.equals(event.prev)
                && Arrays.equals(payloadHash, event.payloadHash);
    }

    @Override
    public int hashCode() {
        return Objects.hash(id, prev, Arrays.hashCode(payloadHash), epoch);
    }
}
