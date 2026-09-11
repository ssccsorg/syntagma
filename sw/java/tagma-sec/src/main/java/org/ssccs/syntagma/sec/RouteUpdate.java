package org.ssccs.syntagma.sec;

import java.util.Objects;

/**
 * Outcome of a successful route update: the record's audit entry and the
 * receipt proving delivery and origin (non-repudiation).
 *
 * <p>Port of the C++ {@code tagma_sec::RouteUpdate} in
 * {@code sw/cpp/tagma_sec/include/tagma_sec/proxy.h}; the underlying behavior
 * mirrors the Rust {@code RouteUpdate} in {@code sw/rust/sec/src/proxy.rs}.
 *
 * @param event   the audit entry committing to the record
 * @param receipt the receipt binding the record to its path, epoch, and
 *                origin principal
 */
public record RouteUpdate(Event event, Receipt receipt) {

    /**
     * @throws NullPointerException when {@code event} or {@code receipt} is null
     */
    public RouteUpdate {
        Objects.requireNonNull(event, "event");
        Objects.requireNonNull(receipt, "receipt");
    }
}
