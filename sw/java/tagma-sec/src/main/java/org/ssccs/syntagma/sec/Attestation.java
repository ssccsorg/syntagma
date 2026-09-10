package org.ssccs.syntagma.sec;

import java.util.Objects;

/**
 * A capability statement binding a principal to a scope, with a lifetime.
 * Authorization succeeds only for epochs inside
 * {@code issuedEpoch..validUntil}, both ends inclusive.
 *
 * <p>Port of the C++ {@code tagma_sec::Attestation} in
 * {@code sw/cpp/tagma_sec/include/tagma_sec/types.h}; the underlying behavior
 * mirrors the Rust {@code Attestation} in {@code sw/rust/sec/src/types.rs}.
 * The C++ field {@code issued_epoch} is {@code issuedEpoch} here.
 *
 * @param id           attestation identity, assigned by the issuing authority
 * @param principal    identity of the principal the attestation was issued to
 * @param scope        the granted scope
 * @param issuedEpoch  first epoch the attestation authorizes
 * @param validUntil   last epoch the attestation authorizes
 */
public record Attestation(long id, long principal, Scope scope, long issuedEpoch, long validUntil) {

    /**
     * @throws NullPointerException when {@code scope} is null
     */
    public Attestation {
        Objects.requireNonNull(scope, "scope");
    }
}
