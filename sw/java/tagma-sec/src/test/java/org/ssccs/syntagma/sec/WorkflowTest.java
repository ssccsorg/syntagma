package org.ssccs.syntagma.sec;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.ssccs.syntagma.sec.Fixtures.bytes;
import static org.ssccs.syntagma.sec.Fixtures.hex;
import static org.ssccs.syntagma.sec.Fixtures.path;
import static org.ssccs.syntagma.sec.Fixtures.withBothStacks;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import org.junit.jupiter.api.Test;
import org.ssccs.syntagma.core.CoordPath;

/**
 * Common scenario tests for the route-update workflow. The same requirements
 * are exercised against both the legacy and the tagma-sec stacks through the
 * {@link SecStack} proxy, making the stack swap the contract under test.
 *
 * <p>Translation of the C++ test suite
 * {@code sw/cpp/tagma_sec/tests/test_workflow.cpp}; the underlying behavior
 * mirrors {@code sw/rust/sec/tests/workflow.rs}.
 */
class WorkflowTest {

    /** Issues an attestation valid from epoch 0 through epoch 100. */
    private static Attestation issueDefault(SecStack stack, long principal, Scope scope) {
        return stack.authority().issue(principal, scope, 0, 100)
                .orElseThrow(() -> new IllegalStateException("issuance failed"));
    }

    @Test
    void authorizedRouteUpdateIsAcceptedAndChained() {
        withBothStacks(stack -> {
            long principal = stack.authority().registerPrincipal(new byte[0]);
            Attestation att = issueDefault(stack, principal, Scope.prefix(path(1, 2)));

            Optional<RouteUpdate> first =
                    SecStack.routeUpdate(stack, att, path(1, 2, 3), bytes("route-v1"), 50);
            assertTrue(first.isPresent(), "in-scope update accepted");
            if (first.isPresent()) {
                assertEquals(0, first.orElseThrow().event().id(), "first event id");
                assertTrue(first.orElseThrow().event().prev().isEmpty(), "first prev none");
            }

            Optional<RouteUpdate> second =
                    SecStack.routeUpdate(stack, att, path(1, 2, 4), bytes("route-v2"), 51);
            assertTrue(second.isPresent(), "second in-scope update accepted");
            if (second.isPresent()) {
                assertEquals(2, second.orElseThrow().event().id(),
                        "receipt entry of the first update takes id 1");
                assertTrue(second.orElseThrow().event().prev().isPresent()
                        && second.orElseThrow().event().prev().getAsLong() == 1,
                        "second prev 1");
            }
            assertTrue(stack.audit().verifyChain(0, second.orElseThrow().event().id()),
                    "chain is intact");
            assertTrue(stack.audit().verifyChain(0, 0), "single-entry chain is intact");
        });
    }

    @Test
    void routeOutsideScopeIsRejectedWithoutAppending() {
        withBothStacks(stack -> {
            long principal = stack.authority().registerPrincipal(new byte[0]);
            Attestation att = issueDefault(stack, principal, Scope.exact(path(1, 2)));

            // A longer path is outside an Exact scope.
            assertFalse(SecStack.routeUpdate(stack, att, path(1, 2, 3), bytes("route-v1"), 50)
                    .isPresent(), "longer path rejected");

            // The exact path is accepted afterwards, and the chain starts clean.
            Optional<RouteUpdate> accepted =
                    SecStack.routeUpdate(stack, att, path(1, 2), bytes("route-v1"), 50);
            assertTrue(accepted.isPresent() && accepted.orElseThrow().event().id() == 0,
                    "exact path accepted");
            assertTrue(stack.audit().verifyChain(0, 1), "record plus receipt chained");
        });
    }

    @Test
    void expiredAttestationIsRejected() {
        withBothStacks(stack -> {
            long principal = stack.authority().registerPrincipal(new byte[0]);
            Optional<Attestation> att =
                    stack.authority().issue(principal, Scope.prefix(path(1, 2)), 0, 10);
            assertTrue(att.isPresent(), "issuance succeeds");
            assertTrue(SecStack.routeUpdate(stack, att.orElseThrow(), path(1, 2, 3),
                    bytes("route-v1"), 10).isPresent(), "valid at the end epoch");
            assertFalse(SecStack.routeUpdate(stack, att.orElseThrow(), path(1, 2, 3),
                    bytes("route-v1"), 11).isPresent(), "expired attestation rejected");
        });
    }

    @Test
    void attestationNotYetValidIsRejected() {
        withBothStacks(stack -> {
            long principal = stack.authority().registerPrincipal(new byte[0]);
            Optional<Attestation> att =
                    stack.authority().issue(principal, Scope.prefix(path(1, 2)), 10, 100);
            assertTrue(att.isPresent(), "issuance succeeds");
            assertFalse(SecStack.routeUpdate(stack, att.orElseThrow(), path(1, 2, 3),
                    bytes("route-v1"), 9).isPresent(), "before issued epoch rejected");
            assertTrue(SecStack.routeUpdate(stack, att.orElseThrow(), path(1, 2, 3),
                    bytes("route-v1"), 10).isPresent(), "valid at the issued epoch");
        });
    }

    @Test
    void malformedAttestationLifetimeNeverAuthorizes() {
        withBothStacks(stack -> {
            long principal = stack.authority().registerPrincipal(new byte[0]);
            // An invalid window: the issued epoch lies after the validity end.
            Optional<Attestation> att =
                    stack.authority().issue(principal, Scope.prefix(path(1, 2)), 100, 10);
            assertTrue(att.isPresent(), "issuance succeeds");
            assertEquals(Decision.DENY, stack.authority().authorize(att.orElseThrow(),
                    path(1, 2, 3), Authority.ACTION_ROUTE_UPDATE, 50),
                    "invalid lifetime window fails closed");
            assertFalse(SecStack.routeUpdate(stack, att.orElseThrow(), path(1, 2, 3),
                    bytes("route-v1"), 50).isPresent(), "invalid window never authorizes");
        });
    }

    @Test
    void principalsAreScopedIndependently() {
        withBothStacks(stack -> {
            long alice = stack.authority().registerPrincipal(new byte[0]);
            long bob = stack.authority().registerPrincipal(new byte[0]);
            Attestation aliceAtt = issueDefault(stack, alice, Scope.prefix(path(1, 2)));
            Attestation bobAtt = issueDefault(stack, bob, Scope.prefix(path(5, 6)));

            assertFalse(SecStack.routeUpdate(stack, aliceAtt, path(5, 6), bytes("route-bob"), 50)
                    .isPresent(), "alice must not act on bob's scope");
            assertFalse(SecStack.routeUpdate(stack, bobAtt, path(1, 2), bytes("route-alice"), 50)
                    .isPresent(), "bob must not act on alice's scope");

            Optional<RouteUpdate> res =
                    SecStack.routeUpdate(stack, aliceAtt, path(1, 2, 3), bytes("route-alice"), 50);
            assertTrue(res.isPresent() && res.orElseThrow().receipt().remote() == alice,
                    "alice update accepted");
        });
    }

    @Test
    void authorizeEnforcesStoredAttestationWindow() {
        withBothStacks(stack -> {
            long principal = stack.authority().registerPrincipal(new byte[0]);
            Optional<Attestation> att =
                    stack.authority().issue(principal, Scope.exact(path(1, 2)), 0, 10);
            assertTrue(att.isPresent(), "issuance succeeds");
            // A presented copy with a widened window must not extend the grant.
            Attestation stored = att.orElseThrow();
            Attestation forged = new Attestation(stored.id(), stored.principal(),
                    stored.scope(), stored.issuedEpoch(), 1000);
            assertEquals(Decision.DENY, stack.authority().authorize(forged, path(1, 2),
                    Authority.ACTION_ROUTE_UPDATE, 11),
                    "authorization uses the stored validity window");
        });
    }

    @Test
    void authorizeEnforcesStoredScopeForRevocation() {
        withBothStacks(stack -> {
            long principal = stack.authority().registerPrincipal(new byte[0]);
            Scope scope = Scope.exact(path(1, 2));
            Optional<Attestation> att = stack.authority().issue(principal, scope, 0, 100);
            assertTrue(att.isPresent(), "issuance succeeds");
            stack.authority().revoke(principal, Scope.exact(path(1, 2)), 10);
            // A presented copy with a different but matching scope must not
            // bypass the revocation of the granted scope.
            Attestation stored = att.orElseThrow();
            Attestation forged = new Attestation(stored.id(), stored.principal(),
                    Scope.prefix(path(1, 2)), stored.issuedEpoch(), stored.validUntil());
            assertEquals(Decision.DENY, stack.authority().authorize(forged, path(1, 2),
                    Authority.ACTION_ROUTE_UPDATE, 10),
                    "authorization consults the stored scope's revocation record");
        });
    }

    @Test
    void revokedScopeStopsAuthorization() {
        withBothStacks(stack -> {
            long principal = stack.authority().registerPrincipal(new byte[0]);
            Scope scope = Scope.prefix(path(1, 2));
            Optional<Attestation> att = stack.authority().issue(principal, scope, 0, 100);
            assertTrue(att.isPresent(), "issuance succeeds");
            assertEquals(Decision.ALLOW, stack.authority().authorize(att.orElseThrow(),
                    path(1, 2, 3), Authority.ACTION_ROUTE_UPDATE, 5), "allow before revocation");
            stack.authority().revoke(principal, Scope.prefix(path(1, 2)), 10);
            assertFalse(SecStack.routeUpdate(stack, att.orElseThrow(), path(1, 2, 3),
                    bytes("route-v1"), 10).isPresent(),
                    "revoked scope stops authorization at the recorded epoch");
        });
    }

    @Test
    void tamperedRecordFailsIntegrityVerification() {
        withBothStacks(stack -> {
            long principal = stack.authority().registerPrincipal(new byte[0]);
            byte[] record = bytes("route-v1");
            CoordPath target = path(1, 2, 3);

            Seal seal = stack.integrity().seal(record, target, principal, 50);
            assertTrue(stack.integrity().verify(record, target, principal, 50, seal),
                    "seal verifies");
            assertFalse(stack.integrity().verify(bytes("route-v1-tampered"), target, principal,
                    50, seal), "altered record fails verification");
        });
    }

    @Test
    void integrityRefreshRebindsSeal() {
        withBothStacks(stack -> {
            long principal = stack.authority().registerPrincipal(new byte[0]);
            byte[] record = bytes("route-v1");
            CoordPath target = path(1, 2);

            Seal seal = stack.integrity().seal(record, target, principal, 5);
            Optional<Seal> refreshed =
                    stack.integrity().refresh(record, target, principal, 5, 6, seal);
            assertTrue(refreshed.isPresent(), "refresh from the binding epoch succeeds");
            if (refreshed.isPresent()) {
                assertTrue(stack.integrity().verify(record, target, principal, 6,
                        refreshed.orElseThrow()), "refreshed seal verifies at the new epoch");
            }
        });
    }

    @Test
    void channelDetectsTamperedEvidence() {
        withBothStacks(stack -> {
            byte[] evidence = bytes("route-update-request");
            SignedEvidence signedEvidence = stack.channel().sign(evidence);
            assertTrue(stack.channel().verify(signedEvidence), "signed evidence verifies");

            byte[] forgedEvidence = signedEvidence.evidence();
            forgedEvidence[0] = (byte) (forgedEvidence[0] ^ 0xFF);
            SignedEvidence forged = new SignedEvidence(forgedEvidence, signedEvidence.tag());
            assertFalse(stack.channel().verify(forged), "altered evidence fails verification");
        });
    }

    @Test
    void channelExchangeBindsEpochAndRemote() {
        withBothStacks(stack -> {
            Receipt receipt = stack.channel().exchange(bytes("request"), 7, 5);
            assertTrue(stack.channel().verifyReceipt(receipt), "receipt verifies");
            assertTrue(receipt.remote() == 7 && receipt.epoch() == 5, "receipt fields");

            Receipt forgedEpoch =
                    new Receipt(receipt.signedEvidence(), receipt.remote(), 6);
            assertFalse(stack.channel().verifyReceipt(forgedEpoch), "epoch tampering detected");

            Receipt forgedRemote =
                    new Receipt(receipt.signedEvidence(), 8, receipt.epoch());
            assertFalse(stack.channel().verifyReceipt(forgedRemote), "remote tampering detected");
        });
    }

    @Test
    void workflowProducesVerifiableReceipt() {
        withBothStacks(stack -> {
            long principal = stack.authority().registerPrincipal(new byte[0]);
            Attestation att = issueDefault(stack, principal, Scope.prefix(path(1, 2)));

            Optional<RouteUpdate> res =
                    SecStack.routeUpdate(stack, att, path(1, 2, 3), bytes("route-v1"), 50);
            assertTrue(res.isPresent(), "update accepted");
            if (res.isPresent()) {
                assertTrue(stack.channel().verifyReceipt(res.orElseThrow().receipt()),
                        "workflow receipt verifies");
                assertTrue(res.orElseThrow().receipt().remote() == principal
                        && res.orElseThrow().receipt().epoch() == 50,
                        "receipt binds origin and epoch");
                Receipt forged = new Receipt(res.orElseThrow().receipt().signedEvidence(),
                        res.orElseThrow().receipt().remote(), 51);
                assertFalse(stack.channel().verifyReceipt(forged), "tampered receipt fails");
            }
        });
    }

    @Test
    void auditChainRejectsInvalidRanges() {
        withBothStacks(stack -> {
            stack.audit().append(bytes("event-0"), 0);
            stack.audit().append(bytes("event-1"), 1);
            stack.audit().append(bytes("event-2"), 2);

            assertTrue(stack.audit().verifyChain(0, 2), "chain intact");
            assertFalse(stack.audit().verifyChain(2, 1), "reversed range rejected");
            assertFalse(stack.audit().verifyChain(0, 3), "range past the end rejected");
        });
    }

    @Test
    void auditProofAndExportVerify() {
        withBothStacks(stack -> {
            stack.audit().append(bytes("event-0"), 0);
            stack.audit().append(bytes("event-1"), 1);
            stack.audit().append(bytes("event-2"), 2);

            Optional<InclusionProof> proof = stack.audit().prove(1);
            assertTrue(proof.isPresent() && proof.orElseThrow().verify(),
                    "inclusion proof verifies");

            Optional<EvidenceBundle> bundle = stack.audit().exportRange(0, 2);
            assertTrue(bundle.isPresent() && bundle.orElseThrow().verify(),
                    "evidence bundle verifies");

            Optional<EvidenceBundle> mid = stack.audit().exportRange(1, 2);
            assertTrue(mid.isPresent() && mid.orElseThrow().verify(),
                    "mid-range bundle verifies despite an external prev link");

            if (bundle.isPresent()) {
                List<Entry> entries = new ArrayList<>(bundle.orElseThrow().entries());
                Entry tampered = entries.get(1);
                entries.set(1, new Entry(tampered.event(), bytes("tampered")));
                assertFalse(new EvidenceBundle(entries).verify(),
                        "tampered payload breaks bundle verification");
            }

            assertFalse(stack.audit().prove(3).isPresent(), "out-of-range proof rejected");
            assertFalse(stack.audit().exportRange(2, 1).isPresent(), "reversed range rejected");
        });
    }

    @Test
    void auditEventCommitsToRecordPayload() {
        withBothStacks(stack -> {
            long principal = stack.authority().registerPrincipal(new byte[0]);
            Attestation att = issueDefault(stack, principal, Scope.prefix(path(1, 2)));
            byte[] record = bytes("route-v1");
            Optional<RouteUpdate> res =
                    SecStack.routeUpdate(stack, att, path(1, 2, 3), record, 50);
            assertTrue(res.isPresent(), "update accepted");
            if (res.isPresent()) {
                // Pinned against an independent OpenSSL computation of
                // sha256("route-v1"); recomputing it with Hashes.sha256 would
                // make the assertion unable to fail on a wrong commitment.
                assertArrayEquals(
                        hex("3e1acf18e9b268eb42f54230cd96c449d2bb6248a13e1d3f55322e7bfd66985e"),
                        res.orElseThrow().event().payloadHash(),
                        "audit entry commits to the pinned record payload");
            }
        });
    }
}
