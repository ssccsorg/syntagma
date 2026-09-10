package org.ssccs.syntagma.sec;

import java.util.Optional;
import org.ssccs.syntagma.core.CoordPath;

/**
 * Authority: principal registration, attestation issuance, authorization, and
 * revocation.
 *
 * <p>Scope matching follows the Milestone 1 contract: Exact and Prefix rules
 * only. Authorization never depends on path secrecy; paths are observable and
 * replayable by design.
 *
 * <p>Port of the C++ {@code tagma_sec::Authority} in
 * {@code sw/cpp/tagma_sec/include/tagma_sec/authority.h}; the underlying
 * behavior mirrors the Rust {@code Authority} trait in
 * {@code sw/rust/sec/src/authority.rs}.
 *
 * <p>Type mapping: the C++ {@code PrincipalId} and {@code Epoch} aliases
 * ({@code uint64_t}) are Java {@code long}, the {@code Action} alias
 * ({@code uint8_t}) is {@code int}, and the {@code Path} alias
 * ({@code std::vector<Coord>}) is {@link CoordPath}.
 */
public interface Authority {

    /**
     * The single action of the PoC workflow: a coordination node route update.
     *
     * <p>Mirrors the C++ {@code ACTION_ROUTE_UPDATE} and the Rust
     * {@code ACTION_ROUTE_UPDATE}.
     */
    int ACTION_ROUTE_UPDATE = 1;

    /**
     * Registers a principal from its credentials and returns its identity.
     *
     * @param cred the principal credentials, empty when the implementation
     *             does not discriminate by credential
     * @return the new principal identity, unique per authority instance
     */
    long registerPrincipal(byte[] cred);

    /**
     * Issues an attestation scoped to {@code scope}, valid from
     * {@code issuedEpoch} through {@code validUntil} inclusive.
     *
     * @return the issued attestation, or empty when the authority declines to
     *         issue
     */
    Optional<Attestation> issue(long principal, Scope scope, long issuedEpoch, long validUntil);

    /**
     * Decides whether {@code att} authorizes {@code path} for {@code action}
     * at {@code epoch}.
     *
     * <p>The action is carried by the interface; the Milestone 1 policy is
     * scope-only, so implementations do not yet discriminate by action.
     */
    Decision authorize(Attestation att, CoordPath path, int action, long epoch);

    /**
     * Revokes the given scope for the principal, effective from {@code epoch}.
     */
    void revoke(long principal, Scope scope, long epoch);
}
