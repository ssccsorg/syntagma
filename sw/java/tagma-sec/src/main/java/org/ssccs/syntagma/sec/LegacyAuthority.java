package org.ssccs.syntagma.sec;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Optional;
import java.util.TreeMap;
import org.ssccs.syntagma.core.CoordPath;

/**
 * Legacy security pattern: an ACL-style registry of attestations per
 * principal.
 *
 * <p>{@code revoke} removes the scope immediately and the epoch argument is
 * ignored, so there is no grace window before the recorded epoch, and a fresh
 * grant restores authorization. This is the baseline the tagma-sec pattern is
 * compared against in the shared workflow tests.
 *
 * <p>Port of the C++ {@code tagma_sec::LegacyAuthority} in
 * {@code sw/cpp/tagma_sec/include/tagma_sec/legacy.h}; the underlying behavior
 * mirrors the Rust {@code LegacyAuthority} in
 * {@code sw/rust/sec/src/legacy.rs}.
 */
public final class LegacyAuthority implements Authority {

    private long nextId;
    private long nextAtt;
    private final Map<Long, List<Attestation>> attestations = new TreeMap<>();

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
        List<Attestation> list = attestations.get(att.principal());
        if (list == null) {
            return Decision.DENY;
        }
        for (Attestation stored : list) {
            if (stored.id() == att.id()
                    && stored.scope().matches(path)
                    && epoch >= stored.issuedEpoch()
                    && epoch <= stored.validUntil()) {
                return Decision.ALLOW;
            }
        }
        return Decision.DENY;
    }

    @Override
    public void revoke(long principal, Scope scope, long epoch) {
        Objects.requireNonNull(scope, "scope");
        List<Attestation> list = attestations.get(principal);
        if (list == null) {
            return;
        }
        list.removeIf(stored -> stored.scope().equals(scope));
    }
}
