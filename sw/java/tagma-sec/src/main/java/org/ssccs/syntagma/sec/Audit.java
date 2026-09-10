package org.ssccs.syntagma.sec;

import java.util.Optional;

/**
 * Audit: append-only chained evidence log.
 *
 * <p>Every entry chains to its predecessor, so retroactive modification of an
 * interior entry breaks the chain. Inclusion proofs and evidence bundles pair
 * each event with its payload so an external party can verify commitments
 * without access to the log.
 *
 * <p>Port of the C++ {@code tagma_sec::Audit} in
 * {@code sw/cpp/tagma_sec/include/tagma_sec/audit.h}; the underlying behavior
 * mirrors the Rust {@code Audit} trait in {@code sw/rust/sec/src/audit.rs}.
 * The C++ {@code export_range} is {@code exportRange} here.
 */
public interface Audit {

    /**
     * Appends an event for {@code payload} at {@code epoch} and returns the
     * appended entry.
     */
    Event append(byte[] payload, long epoch);

    /**
     * Returns true when the chain is intact over entries {@code from..=to}.
     */
    boolean verifyChain(long from, long to);

    /**
     * Returns an inclusion proof for {@code entry}, or empty when out of
     * range.
     */
    Optional<InclusionProof> prove(long entry);

    /**
     * Returns an exportable evidence bundle over entries {@code from..=to},
     * or empty when the range is invalid.
     */
    Optional<EvidenceBundle> exportRange(long from, long to);
}
