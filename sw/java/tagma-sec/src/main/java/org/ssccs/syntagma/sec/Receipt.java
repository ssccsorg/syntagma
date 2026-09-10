package org.ssccs.syntagma.sec;

import java.util.Objects;

/**
 * Proof of delivery and origin for exchanged evidence: the signed evidence
 * plus the remote principal and the epoch it binds.
 *
 * <p>Port of the C++ {@code tagma_sec::Receipt} in
 * {@code sw/cpp/tagma_sec/include/tagma_sec/types.h}; the underlying behavior
 * mirrors the Rust {@code Receipt} in {@code sw/rust/sec/src/types.rs}. The
 * C++ field {@code signed_evidence} (Rust {@code signed}) is
 * {@code signedEvidence} here.
 *
 * @param signedEvidence the signed evidence binding the exchange
 * @param remote         the remote principal the evidence was exchanged with
 * @param epoch          the epoch the exchange is bound to
 */
public record Receipt(SignedEvidence signedEvidence, long remote, long epoch) {

    /**
     * @throws NullPointerException when {@code signedEvidence} is null
     */
    public Receipt {
        Objects.requireNonNull(signedEvidence, "signedEvidence");
    }
}
