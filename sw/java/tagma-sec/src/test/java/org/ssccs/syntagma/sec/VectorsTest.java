package org.ssccs.syntagma.sec;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.ssccs.syntagma.sec.Fixtures.bytes;
import static org.ssccs.syntagma.sec.Fixtures.hex;
import static org.ssccs.syntagma.sec.Fixtures.path;

import java.util.Optional;
import org.junit.jupiter.api.Test;

/**
 * Module-level tag and commitment vectors pinned against independent OpenSSL
 * computations.
 *
 * <p>Each value was computed with OpenSSL from the layout the module documents
 * (key literal, part order, little-endian encodings), not from this code. A
 * drift in a key literal, a part order, or a length encoding therefore fails
 * here and in the C++ mirror {@code sw/cpp/tagma_sec/tests/test_vectors.cpp}
 * instead of passing silently: the workflow tests only recompute and compare,
 * so they cannot see such a change.
 */
class VectorsTest {

    @Test
    void legacySealTagIsPinned() {
        LegacyIntegrity integrity = new LegacyIntegrity();
        Seal seal = integrity.seal(bytes("route-v1"), path(1, 2), 7, 5);
        assertArrayEquals(hex("3ea058c754d3e7a619ebc94ea9e08bdfb24c69349c1a71c8a5e453b6516ccc0f"),
                seal.tag(), "legacy seal tag matches the openssl vector");
    }

    @Test
    void delosSealTagIsPinned() {
        DelosIntegrity integrity = new DelosIntegrity();
        Seal seal = integrity.seal(bytes("route-v1"), path(1, 2), 7, 5);
        assertArrayEquals(hex("fd70c7dd4ef7652099a8feef0e86d62ddd572fdcf545e32ce6ae2caede0a56f9"),
                seal.tag(), "delos seal tag matches the openssl vector");
    }

    @Test
    void channelTagIsPinned() {
        MacChannel channel = new MacChannel();
        SignedEvidence signed = channel.sign(bytes("route-update-request"));
        assertArrayEquals(hex("d9242ea0622659adea42adc0e7a1cdbe70355c4ca7bff008ddd33f8ebad8c7b4"),
                signed.tag(), "channel tag matches the openssl vector");
    }

    @Test
    void auditCommitmentIsPinned() {
        ChainedAudit audit = new ChainedAudit();
        Event event = audit.append(bytes("event-0"), 0);
        assertArrayEquals(hex("23352a67ac7ffc4b5d98023dc1a54ee507d4bf95dc1c4154045c5e2bbde73317"),
                event.payloadHash(), "audit commitment matches the openssl vector");
    }

    @Test
    void workflowReceiptTagIsPinned() {
        SecStack stack = SecStack.delos();
        long principal = stack.authority().registerPrincipal(new byte[0]);
        Optional<Attestation> att =
                stack.authority().issue(principal, Scope.prefix(path(1, 2)), 0, 100);
        assertTrue(att.isPresent(), "issuance succeeds");
        Optional<RouteUpdate> res = SecStack.routeUpdate(stack, att.orElseThrow(), path(1, 2, 3),
                bytes("route-v1"), 50);
        assertTrue(res.isPresent(), "update accepted");
        if (res.isPresent()) {
            // The receipt tag is the channel tag over the concatenation
            // routeUpdate exchanges: record | le16 path | le64 epoch | le64
            // remote | le64 epoch.
            assertArrayEquals(hex("ce8ef6f248b2426f1900821041662ef4d42821e1e77c0ad6479d0cec07f319c5"),
                    res.orElseThrow().receipt().signedEvidence().tag(),
                    "workflow receipt tag matches the openssl vector");
            assertEquals(principal, res.orElseThrow().receipt().remote(),
                    "the pinned receipt binds the first registered principal");
            assertEquals(50, res.orElseThrow().receipt().epoch(),
                    "the pinned receipt binds epoch 50");
        }
    }
}
