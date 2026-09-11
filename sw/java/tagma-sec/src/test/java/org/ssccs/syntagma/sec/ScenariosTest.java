package org.ssccs.syntagma.sec;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.ssccs.syntagma.sec.Fixtures.bytes;
import static org.ssccs.syntagma.sec.Fixtures.path;
import static org.ssccs.syntagma.sec.Fixtures.withBothStacks;

import java.util.Arrays;
import java.util.Optional;
import org.junit.jupiter.api.Test;
import org.ssccs.syntagma.core.Coord;
import org.ssccs.syntagma.core.CoordPath;

/**
 * Real-scenario tests for the route-update workflow. These pin the expected
 * behavior of complete journeys and investigation flows. The expectation is
 * the contract: a failure means the implementation must change, the test
 * stays. Every scenario runs on both stacks through the mirror pattern.
 *
 * <p>Translation of the C++ test suite
 * {@code sw/cpp/tagma_sec/tests/test_scenarios.cpp}; the underlying behavior
 * mirrors {@code sw/rust/sec/tests/scenarios.rs}.
 */
class ScenariosTest {

    @Test
    void routeUpdateJourneyIsIdenticalOnBothStacks() {
        withBothStacks(stack -> {
            long alice = stack.authority().registerPrincipal(new byte[0]);
            long bob = stack.authority().registerPrincipal(new byte[0]);
            Scope aliceScope = Scope.prefix(path(1, 2));
            Optional<Attestation> aliceAtt =
                    stack.authority().issue(alice, aliceScope, 0, 60);
            Optional<Attestation> bobAtt =
                    stack.authority().issue(bob, Scope.exact(path(9, 9)), 0, 100);
            assertTrue(aliceAtt.isPresent() && bobAtt.isPresent(), "issuance succeeds");

            Optional<RouteUpdate> r1 =
                    SecStack.routeUpdate(stack, aliceAtt.orElseThrow(), path(1, 2, 3), bytes("r1"), 10);
            assertTrue(r1.isPresent() && r1.orElseThrow().event().id() == 0
                    && r1.orElseThrow().receipt().remote() == alice, "alice update in scope");

            // Cross-principal denial: bob cannot act in alice's subtree.
            assertFalse(SecStack.routeUpdate(stack, bobAtt.orElseThrow(), path(1, 2, 4),
                    bytes("r2"), 11).isPresent(), "bob denied in alice's subtree");

            // Mid-journey revocation: alice loses the scope at epoch 20.
            stack.authority().revoke(alice, Scope.prefix(path(1, 2)), 20);
            assertFalse(SecStack.routeUpdate(stack, aliceAtt.orElseThrow(), path(1, 2, 5),
                    bytes("r3"), 21).isPresent(), "alice denied after revocation");

            // Expiry: bob's attestation dies at epoch 100.
            assertFalse(SecStack.routeUpdate(stack, bobAtt.orElseThrow(), path(9, 9),
                    bytes("r4"), 101).isPresent(), "bob denied after expiry");

            // Denials append nothing: the log holds exactly the two entries of
            // the one accepted update.
            assertTrue(stack.audit().verifyChain(0, 1), "chain covers the accepted update");
            assertFalse(stack.audit().prove(2).isPresent(), "denials append no events");
            Optional<EvidenceBundle> bundle = stack.audit().exportRange(0, 1);
            assertTrue(bundle.isPresent() && bundle.orElseThrow().verify(),
                    "evidence bundle verifies");
        });
    }

    @Test
    void auditInvestigationFlowProvesWhoDidWhat() {
        withBothStacks(stack -> {
            long alice = stack.authority().registerPrincipal(new byte[0]);
            long bob = stack.authority().registerPrincipal(new byte[0]);
            Optional<Attestation> aliceAtt =
                    stack.authority().issue(alice, Scope.prefix(path(1, 2)), 0, 100);
            Optional<Attestation> bobAtt =
                    stack.authority().issue(bob, Scope.exact(path(7, 8)), 0, 100);
            assertTrue(aliceAtt.isPresent() && bobAtt.isPresent(), "issuance succeeds");

            Optional<RouteUpdate> a1 = SecStack.routeUpdate(stack, aliceAtt.orElseThrow(),
                    path(1, 2, 3), bytes("route-a"), 10);
            Optional<RouteUpdate> b1 = SecStack.routeUpdate(stack, bobAtt.orElseThrow(),
                    path(7, 8), bytes("route-b"), 12);
            assertTrue(a1.isPresent() && b1.isPresent(), "both updates accepted");

            // Offline investigation: the exported bundle verifies without the
            // log, and each receipt binds its origin and its epoch.
            Optional<EvidenceBundle> bundle = stack.audit().exportRange(0, 3);
            assertTrue(bundle.isPresent() && bundle.orElseThrow().verify(),
                    "full evidence bundle verifies");

            assertTrue(stack.channel().verifyReceipt(a1.orElseThrow().receipt()),
                    "alice receipt verifies");
            assertTrue(a1.orElseThrow().receipt().remote() == alice
                    && a1.orElseThrow().receipt().epoch() == 10, "alice receipt fields");
            assertTrue(stack.channel().verifyReceipt(b1.orElseThrow().receipt()),
                    "bob receipt verifies");
            assertTrue(b1.orElseThrow().receipt().remote() == bob
                    && b1.orElseThrow().receipt().epoch() == 12, "bob receipt fields");

            // Replay resistance: a later update produces different signed
            // evidence, so a captured artifact is stale by construction.
            Optional<RouteUpdate> a2 = SecStack.routeUpdate(stack, aliceAtt.orElseThrow(),
                    path(1, 2, 4), bytes("route-a2"), 20);
            assertTrue(a2.isPresent(), "second alice update accepted");
            if (a2.isPresent()) {
                assertNotEquals(a2.orElseThrow().receipt().signedEvidence(),
                        a1.orElseThrow().receipt().signedEvidence(),
                        "evidence differs across updates");
                assertFalse(Arrays.equals(a2.orElseThrow().receipt().signedEvidence().tag(),
                        a1.orElseThrow().receipt().signedEvidence().tag()),
                        "tags differ across updates");
                assertTrue(a2.orElseThrow().receipt().epoch() == 20,
                        "second receipt binds its epoch");
            }
        });
    }

    @Test
    void singleEpochWindowIsExact() {
        withBothStacks(stack -> {
            long principal = stack.authority().registerPrincipal(new byte[0]);
            Optional<Attestation> att =
                    stack.authority().issue(principal, Scope.prefix(path(1, 2)), 10, 10);
            assertTrue(att.isPresent(), "issuance succeeds");

            assertEquals(Decision.DENY, stack.authority().authorize(att.orElseThrow(),
                    path(1, 2, 3), Authority.ACTION_ROUTE_UPDATE, 9),
                    "before the single epoch denied");
            assertEquals(Decision.ALLOW, stack.authority().authorize(att.orElseThrow(),
                    path(1, 2, 3), Authority.ACTION_ROUTE_UPDATE, 10),
                    "the single epoch is valid");
            assertEquals(Decision.DENY, stack.authority().authorize(att.orElseThrow(),
                    path(1, 2, 3), Authority.ACTION_ROUTE_UPDATE, 11),
                    "after the single epoch denied");
        });
    }

    @Test
    void emptyPrefixScopeAuthorizesEveryPath() {
        withBothStacks(stack -> {
            long principal = stack.authority().registerPrincipal(new byte[0]);
            Optional<Attestation> att =
                    stack.authority().issue(principal, Scope.prefix(path()), 0, 100);
            assertTrue(att.isPresent(), "issuance succeeds");

            assertEquals(Decision.ALLOW, stack.authority().authorize(att.orElseThrow(),
                    path(1, 2, 3), Authority.ACTION_ROUTE_UPDATE, 50),
                    "an empty prefix scope is the root scope");
            assertEquals(Decision.ALLOW, stack.authority().authorize(att.orElseThrow(),
                    path(7, 8), Authority.ACTION_ROUTE_UPDATE, 50), "root scope covers any path");
        });
    }

    @Test
    void deepPathScenarioReachesTheDepthBound() {
        withBothStacks(stack -> {
            long principal = stack.authority().registerPrincipal(new byte[0]);
            Optional<Attestation> att =
                    stack.authority().issue(principal, Scope.prefix(path(0)), 0, 100);
            assertTrue(att.isPresent(), "issuance succeeds");

            // 19 coords matches the maximum scope depth in the specification.
            Coord[] deepCoords = new Coord[19];
            for (int i = 0; i < deepCoords.length; i++) {
                deepCoords[i] = Coord.fromIndex(i).orElseThrow();
            }
            CoordPath deep = CoordPath.fromArray(deepCoords);
            Optional<RouteUpdate> res =
                    SecStack.routeUpdate(stack, att.orElseThrow(), deep, bytes("deep-route"), 50);
            assertTrue(res.isPresent() && res.orElseThrow().receipt().epoch() == 50,
                    "deep path update accepted");
            assertTrue(stack.audit().verifyChain(0, 1), "deep path chain intact");
        });
    }
}
