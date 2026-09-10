package org.ssccs.syntagma.sec;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Optional;
import java.util.TreeMap;
import org.ssccs.syntagma.core.CoordPath;

/**
 * tagma-sec (Delos) authority: scopes over path sequences with an
 * epoch-scoped revocation list.
 *
 * <p>{@code authorize} reads the window, the scope, and the revocation record
 * from the stored attestation, so a presented copy with a widened window or a
 * different but matching scope cannot extend or reshape the grant. Revocation
 * is recorded per (principal, scope) pair and takes effect from the recorded
 * epoch onward; it persists across re-issuance for the same scope.
 *
 * <p>Port of the C++ {@code tagma_sec::DelosAuthority} in
 * {@code sw/cpp/tagma_sec/include/tagma_sec/delos.h}; the underlying behavior
 * mirrors the Rust {@code DelosAuthority} in {@code sw/rust/sec/src/delos.rs}.
 */
public final class DelosAuthority implements Authority {

    /** Revocation record key: a principal plus the marker-prefixed scope key. */
    private record RevocationKey(long principal, List<Integer> scopeKey) {
    }

    private long nextId;
    private long nextAtt;
    private final Map<Long, List<Attestation>> attestations = new TreeMap<>();
    private final Map<RevocationKey, Long> revoked = new HashMap<>();

    @Override
    public long registerPrincipal(byte[] cred) {
        Objects.requireNonNull(cred, "cred");
        return ++nextId;
    }

    @Override
    public Optional<Attestation> issue(long principal, Scope scope, long issuedEpoch, long validUntil) {
        Objects.requireNonNull(scope, "scope");
        Attestation att = new Attestation(++nextAtt, principal, scope, issuedEpoch, validUntil);
        attestations.computeIfAbsent(principal, key -> new ArrayList<>()).add(att);
        return Optional.of(att);
    }

    @Override
    public Decision authorize(Attestation att, CoordPath path, int action, long epoch) {
        Objects.requireNonNull(att, "att");
        Objects.requireNonNull(path, "path");
        // The presented attestation carries the holder's identity claim; the
        // window, the scope, and the revocation record are read from the
        // stored attestation, so a presented copy cannot extend or reshape
        // the grant.
        List<Attestation> list = attestations.get(att.principal());
        if (list == null) {
            return Decision.DENY;
        }
        Attestation stored = null;
        for (Attestation candidate : list) {
            if (candidate.id() == att.id()) {
                stored = candidate;
                break;
            }
        }
        if (stored == null) {
            return Decision.DENY;
        }
        if (!stored.scope().matches(path)) {
            return Decision.DENY;
        }
        if (epoch < stored.issuedEpoch() || epoch > stored.validUntil()) {
            return Decision.DENY;
        }
        Long revokedAt = revoked.get(new RevocationKey(att.principal(), stored.scope().key()));
        if (revokedAt != null && epoch >= revokedAt) {
            return Decision.DENY;
        }
        return Decision.ALLOW;
    }

    @Override
    public void revoke(long principal, Scope scope, long epoch) {
        Objects.requireNonNull(scope, "scope");
        revoked.put(new RevocationKey(principal, scope.key()), epoch);
    }
}
