package org.ssccs.syntagma.sec;

import java.util.Arrays;
import java.util.Objects;

/**
 * An event paired with the payload it commits to, the unit carried by
 * inclusion proofs and evidence bundles.
 *
 * <p>Port of the C++ {@code tagma_sec::Entry} in
 * {@code sw/cpp/tagma_sec/include/tagma_sec/audit.h}; the underlying behavior
 * mirrors the Rust {@code Entry} in {@code sw/rust/sec/src/audit.rs}. The
 * payload is copied on the way in and cloned on the way out.
 *
 * @param event   the chained audit event
 * @param payload the payload the event commits to
 */
public record Entry(Event event, byte[] payload) {

    /**
     * Compact canonical constructor: stores a defensive copy of the payload.
     */
    public Entry {
        Objects.requireNonNull(event, "event");
        Objects.requireNonNull(payload, "payload");
        payload = payload.clone();
    }

    /** The payload as a defensive copy. */
    @Override
    public byte[] payload() {
        return payload.clone();
    }

    @Override
    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        if (!(other instanceof Entry entry)) {
            return false;
        }
        return event.equals(entry.event) && Arrays.equals(payload, entry.payload);
    }

    @Override
    public int hashCode() {
        return 31 * event.hashCode() + Arrays.hashCode(payload);
    }
}
