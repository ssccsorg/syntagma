package org.ssccs.syntagma.sec;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.ssccs.syntagma.sec.Fixtures.bytes;
import static org.ssccs.syntagma.sec.Fixtures.path;

import java.util.Arrays;
import java.util.List;
import java.util.Optional;
import org.junit.jupiter.api.Test;
import org.ssccs.syntagma.core.CoordPath;

/**
 * tagma-sec-specific properties of the Delos pattern stack. These tests pin
 * the properties that distinguish the tagma-sec pattern from the legacy one:
 * epoch-bound seals, prefix scope semantics, and revocation that is effective
 * only from the recorded epoch.
 *
 * <p>Translation of the C++ test suite
 * {@code sw/cpp/tagma_sec/tests/test_delos.cpp}; the underlying behavior
 * mirrors {@code sw/rust/sec/tests/delos.rs}.
 */
class DelosTest {

    @Test
    void delosSealBindsEpochAndPrincipal() {
        byte[] record = bytes("route-v1");
        CoordPath target = path(1, 2);

        // Legacy seal binds only record and path: principal and epoch are free.
        SecStack legacy = SecStack.legacy();
        Seal legacySeal = legacy.integrity().seal(record, target, 7, 5);
        assertTrue(legacy.integrity().verify(record, target, 7, 5, legacySeal), "legacy verifies");
        assertTrue(legacy.integrity().verify(record, target, 99, 99, legacySeal),
                "legacy seal ignores principal and epoch");

        // Delos seal binds record, path, principal, and epoch.
        SecStack delos = SecStack.delos();
        Seal delosSeal = delos.integrity().seal(record, target, 7, 5);
        assertTrue(delos.integrity().verify(record, target, 7, 5, delosSeal), "delos verifies");
        assertFalse(delos.integrity().verify(record, target, 7, 6, delosSeal),
                "epoch change breaks the delos seal");
        assertFalse(delos.integrity().verify(record, target, 8, 5, delosSeal),
                "principal change breaks the delos seal");
        assertFalse(delos.integrity().verify(record, path(1, 3), 7, 5, delosSeal),
                "path change breaks the delos seal");
    }

    @Test
    void delosRefreshRejectsNonNewerEpoch() {
        SecStack delos = SecStack.delos();
        byte[] record = bytes("route-v1");
        CoordPath target = path(1, 2);
        Seal seal = delos.integrity().seal(record, target, 7, 5);

        assertFalse(delos.integrity().refresh(record, target, 7, 5, 4, seal).isPresent(),
                "refresh to an older epoch rejected");
        assertFalse(delos.integrity().refresh(record, target, 7, 5, 5, seal).isPresent(),
                "refresh to the same epoch rejected");
        assertTrue(delos.integrity().refresh(record, target, 7, 5, 6, seal).isPresent(),
                "refresh to a newer epoch succeeds");
    }

    @Test
    void delosSealChangesWhenAnyBoundInputChanges() {
        SecStack delos = SecStack.delos();
        byte[] record = bytes("route-v1");
        CoordPath target = path(1, 2);
        Seal base = delos.integrity().seal(record, target, 7, 5);

        record Case(byte[] record, CoordPath path, long principal, long epoch) {
        }
        List<Case> cases = List.of(
                new Case(bytes("route-v2"), path(1, 2), 7, 5), // record change
                new Case(record, path(1, 3), 7, 5),            // path change
                new Case(record, path(1, 2), 8, 5),            // principal change
                new Case(record, path(1, 2), 7, 6));           // epoch change
        for (Case bound : cases) {
            Seal seal = delos.integrity().seal(bound.record(), bound.path(),
                    bound.principal(), bound.epoch());
            assertFalse(Arrays.equals(seal.tag(), base.tag()),
                    "delos seal changes when any bound input changes");
        }
    }

    @Test
    void refreshEpochBindingDiffersFromLegacy() {
        byte[] record = bytes("route-v1");
        CoordPath target = path(1, 2);

        // Legacy refresh accepts any from_epoch and re-emits the identical tag.
        SecStack legacy = SecStack.legacy();
        Seal legacySeal = legacy.integrity().seal(record, target, 7, 5);
        Optional<Seal> legacyRefreshed =
                legacy.integrity().refresh(record, target, 7, 99, 100, legacySeal);
        assertTrue(legacyRefreshed.isPresent()
                && Arrays.equals(legacyRefreshed.orElseThrow().tag(), legacySeal.tag()),
                "legacy seal is epoch-free, refresh is a no-op");
        assertTrue(legacy.integrity().refresh(record, target, 7, 5, 4, legacySeal).isPresent(),
                "legacy refresh stays epoch-free even toward an older epoch");

        // Delos refresh re-binds the seal to the new epoch.
        SecStack delos = SecStack.delos();
        Seal delosSeal = delos.integrity().seal(record, target, 7, 5);
        Optional<Seal> delosRefreshed =
                delos.integrity().refresh(record, target, 7, 5, 6, delosSeal);
        assertTrue(delosRefreshed.isPresent(), "delos refresh from the binding epoch succeeds");
        if (delosRefreshed.isPresent()) {
            assertFalse(Arrays.equals(delosRefreshed.orElseThrow().tag(), delosSeal.tag()),
                    "delos refresh re-binds the epoch");
            assertTrue(delos.integrity().verify(record, target, 7, 6,
                    delosRefreshed.orElseThrow()), "refreshed seal verifies at the new epoch");
            assertFalse(delos.integrity().verify(record, target, 7, 5,
                    delosRefreshed.orElseThrow()),
                    "refreshed seal no longer verifies at the old epoch");
        }
        assertFalse(delos.integrity().refresh(record, target, 7, 99, 6, delosSeal).isPresent(),
                "refresh from a wrong epoch rejected");
    }

    @Test
    void prefixScopeCoversLongerPaths() {
        SecStack stack = SecStack.delos();
        long principal = stack.authority().registerPrincipal(new byte[0]);
        Optional<Attestation> att =
                stack.authority().issue(principal, Scope.prefix(path(1, 2)), 0, 100);
        assertTrue(att.isPresent(), "issuance succeeds");
        assertEquals(Decision.ALLOW, stack.authority().authorize(att.orElseThrow(), path(1, 2),
                Authority.ACTION_ROUTE_UPDATE, 50), "prefix matches the scope path itself");
        assertEquals(Decision.ALLOW, stack.authority().authorize(att.orElseThrow(), path(1, 2, 3),
                Authority.ACTION_ROUTE_UPDATE, 50), "prefix matches a longer path");
        assertEquals(Decision.ALLOW,
                stack.authority().authorize(att.orElseThrow(), path(1, 2, 3, 4),
                        Authority.ACTION_ROUTE_UPDATE, 50),
                "prefix matches a much longer path");
        assertEquals(Decision.DENY, stack.authority().authorize(att.orElseThrow(), path(1, 3),
                Authority.ACTION_ROUTE_UPDATE, 50), "sibling path denied");
        assertEquals(Decision.DENY, stack.authority().authorize(att.orElseThrow(), path(2, 3),
                Authority.ACTION_ROUTE_UPDATE, 50), "unrelated path denied");
    }

    @Test
    void exactScopeMatchesOnlyIdenticalPath() {
        SecStack stack = SecStack.delos();
        long principal = stack.authority().registerPrincipal(new byte[0]);
        Optional<Attestation> att =
                stack.authority().issue(principal, Scope.exact(path(1, 2)), 0, 100);
        assertTrue(att.isPresent(), "issuance succeeds");
        assertEquals(Decision.ALLOW, stack.authority().authorize(att.orElseThrow(), path(1, 2),
                Authority.ACTION_ROUTE_UPDATE, 50), "exact path allowed");
        assertEquals(Decision.DENY, stack.authority().authorize(att.orElseThrow(), path(1, 2, 3),
                Authority.ACTION_ROUTE_UPDATE, 50), "longer path denied");
        assertEquals(Decision.DENY, stack.authority().authorize(att.orElseThrow(), path(1, 3),
                Authority.ACTION_ROUTE_UPDATE, 50), "different path denied");
    }

    @Test
    void delosRevocationIsEffectiveFromRecordedEpoch() {
        SecStack stack = SecStack.delos();
        long principal = stack.authority().registerPrincipal(new byte[0]);
        Scope scope = Scope.prefix(path(1, 2));
        Optional<Attestation> att = stack.authority().issue(principal, scope, 0, 100);
        assertTrue(att.isPresent(), "issuance succeeds");
        stack.authority().revoke(principal, Scope.prefix(path(1, 2)), 10);

        assertEquals(Decision.ALLOW, stack.authority().authorize(att.orElseThrow(), path(1, 2, 3),
                Authority.ACTION_ROUTE_UPDATE, 9),
                "revocation not effective before the recorded epoch");
        assertEquals(Decision.DENY, stack.authority().authorize(att.orElseThrow(), path(1, 2, 3),
                Authority.ACTION_ROUTE_UPDATE, 10),
                "revocation takes effect at the recorded epoch");
        assertEquals(Decision.DENY, stack.authority().authorize(att.orElseThrow(), path(1, 2, 3),
                Authority.ACTION_ROUTE_UPDATE, 11),
                "revocation persists after the recorded epoch");

        // Legacy contrast: the ACL entry is removed immediately.
        SecStack legacy = SecStack.legacy();
        long legacyPrincipal = legacy.authority().registerPrincipal(new byte[0]);
        Optional<Attestation> legacyAtt = legacy.authority()
                .issue(legacyPrincipal, Scope.prefix(path(1, 2)), 0, 100);
        assertTrue(legacyAtt.isPresent(), "issuance succeeds");
        legacy.authority().revoke(legacyPrincipal, Scope.prefix(path(1, 2)), 10);
        assertEquals(Decision.DENY, legacy.authority().authorize(legacyAtt.orElseThrow(),
                path(1, 2, 3), Authority.ACTION_ROUTE_UPDATE, 9),
                "legacy revocation removes the entry immediately");
    }

    @Test
    void revocationKeyIsScopeSpecific() {
        SecStack stack = SecStack.delos();
        long principal = stack.authority().registerPrincipal(new byte[0]);
        Optional<Attestation> exactAtt =
                stack.authority().issue(principal, Scope.exact(path(1, 2)), 0, 100);
        Optional<Attestation> prefixAtt =
                stack.authority().issue(principal, Scope.prefix(path(1, 2)), 0, 100);
        assertTrue(exactAtt.isPresent() && prefixAtt.isPresent(), "issuance succeeds");

        stack.authority().revoke(principal, Scope.exact(path(1, 2)), 10);

        assertEquals(Decision.DENY, stack.authority().authorize(exactAtt.orElseThrow(),
                path(1, 2), Authority.ACTION_ROUTE_UPDATE, 10), "exact scope is revoked");
        assertEquals(Decision.ALLOW, stack.authority().authorize(prefixAtt.orElseThrow(),
                path(1, 2, 3), Authority.ACTION_ROUTE_UPDATE, 10),
                "prefix scope has a distinct revocation key");
    }

    @Test
    void attestationLifetimeBoundaryIsInclusive() {
        SecStack stack = SecStack.delos();
        long principal = stack.authority().registerPrincipal(new byte[0]);
        Optional<Attestation> att =
                stack.authority().issue(principal, Scope.prefix(path(1, 2)), 0, 10);
        assertTrue(att.isPresent(), "issuance succeeds");
        assertEquals(Decision.ALLOW, stack.authority().authorize(att.orElseThrow(), path(1, 2, 3),
                Authority.ACTION_ROUTE_UPDATE, 10), "the validity end epoch is inclusive");
        assertEquals(Decision.DENY, stack.authority().authorize(att.orElseThrow(), path(1, 2, 3),
                Authority.ACTION_ROUTE_UPDATE, 11), "the epoch after the window is denied");
    }

    @Test
    void revokedScopeStaysRevokedAfterReissue() {
        // Delos: revocation is (principal, scope)-level and persists, per the
        // specification's "revoked (principal, scope) pair" record.
        SecStack stack = SecStack.delos();
        long principal = stack.authority().registerPrincipal(new byte[0]);
        Scope scope = Scope.prefix(path(1, 2));
        Optional<Attestation> first = stack.authority().issue(principal, scope, 0, 100);
        assertTrue(first.isPresent(), "issuance succeeds");
        stack.authority().revoke(principal, Scope.prefix(path(1, 2)), 20);

        Optional<Attestation> reissued =
                stack.authority().issue(principal, Scope.prefix(path(1, 2)), 21, 100);
        assertTrue(reissued.isPresent(), "re-issuance succeeds");
        assertNotEquals(first.orElseThrow().id(), reissued.orElseThrow().id(),
                "re-issuance assigns a new id");
        assertEquals(Decision.DENY, stack.authority().authorize(reissued.orElseThrow(),
                path(1, 2, 3), Authority.ACTION_ROUTE_UPDATE, 22),
                "a re-issued attestation for a revoked scope stays revoked");
        assertEquals(Decision.ALLOW, stack.authority().authorize(first.orElseThrow(), path(1, 2, 3),
                Authority.ACTION_ROUTE_UPDATE, 19),
                "revocation is effective only from the recorded epoch");

        // Legacy contrast: revocation removes the entry, and a fresh grant
        // restores authorization.
        SecStack legacy = SecStack.legacy();
        long legacyPrincipal = legacy.authority().registerPrincipal(new byte[0]);
        legacy.authority().issue(legacyPrincipal, Scope.prefix(path(1, 2)), 0, 100);
        legacy.authority().revoke(legacyPrincipal, Scope.prefix(path(1, 2)), 20);
        Optional<Attestation> legacyReissued =
                legacy.authority().issue(legacyPrincipal, Scope.prefix(path(1, 2)), 21, 100);
        assertTrue(legacyReissued.isPresent(), "legacy re-issuance succeeds");
        assertEquals(Decision.ALLOW, legacy.authority().authorize(legacyReissued.orElseThrow(),
                path(1, 2, 3), Authority.ACTION_ROUTE_UPDATE, 22),
                "legacy revocation removes entries and permits a fresh grant");
    }
}
