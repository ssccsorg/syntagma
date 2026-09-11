package org.ssccs.syntagma.sec;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.ssccs.syntagma.sec.Fixtures.bytes;

import java.util.OptionalLong;
import org.junit.jupiter.api.Test;

/**
 * Java-specific hazards that the fixed-width C++ types eliminate: tag lengths
 * are runtime values here, and arrays are references rather than values, so
 * every tag and payload is validated and copied on the way in and cloned on
 * the way out.
 *
 * <p>Key-length validation of {@link Hashes#keyedTag} is covered by
 * {@link HashesTest#hashHelpersRejectOutOfDomainInputs()}.
 */
class ValueSemanticsTest {

    @Test
    void fixedLengthTagsAreValidatedOnConstruction() {
        assertThrows(IllegalArgumentException.class, () -> new Seal(new byte[31]),
                "31-byte seal tag rejected");
        assertThrows(IllegalArgumentException.class, () -> new Seal(new byte[33]),
                "33-byte seal tag rejected");
        assertThrows(IllegalArgumentException.class,
                () -> new Event(0, OptionalLong.empty(), new byte[31], 0),
                "31-byte payload hash rejected");
        assertThrows(IllegalArgumentException.class,
                () -> new Event(0, OptionalLong.empty(), new byte[33], 0),
                "33-byte payload hash rejected");
        assertThrows(IllegalArgumentException.class,
                () -> new SignedEvidence(new byte[0], new byte[31]),
                "31-byte evidence tag rejected");
        assertThrows(IllegalArgumentException.class,
                () -> new SignedEvidence(new byte[0], new byte[33]),
                "33-byte evidence tag rejected");
    }

    @Test
    void sealCopiesInAndClonesOut() {
        byte[] tag = new byte[32];
        tag[0] = 1;
        byte[] original = tag.clone();

        Seal seal = new Seal(tag);
        tag[0] = 2;
        assertArrayEquals(original, seal.tag(),
                "caller mutation after construction does not change the seal");

        byte[] fromAccessor = seal.tag();
        fromAccessor[0] = 3;
        assertArrayEquals(original, seal.tag(),
                "mutating an accessor result does not change the seal");
    }

    @Test
    void eventCopiesInAndClonesOut() {
        byte[] payloadHash = new byte[32];
        payloadHash[0] = 1;
        byte[] original = payloadHash.clone();

        Event event = new Event(0, OptionalLong.empty(), payloadHash, 0);
        payloadHash[0] = 2;
        assertArrayEquals(original, event.payloadHash(),
                "caller mutation after construction does not change the commitment");

        byte[] fromAccessor = event.payloadHash();
        fromAccessor[0] = 3;
        assertArrayEquals(original, event.payloadHash(),
                "mutating an accessor result does not change the commitment");
    }

    @Test
    void signedEvidenceCopiesInAndClonesOut() {
        byte[] evidence = bytes("evidence");
        byte[] originalEvidence = evidence.clone();
        byte[] tag = new byte[32];
        tag[0] = 1;
        byte[] originalTag = tag.clone();

        SignedEvidence signed = new SignedEvidence(evidence, tag);
        evidence[0] = 'X';
        tag[0] = 2;
        assertArrayEquals(originalEvidence, signed.evidence(),
                "caller mutation after construction does not change the evidence");
        assertArrayEquals(originalTag, signed.tag(),
                "caller mutation after construction does not change the tag");

        signed.evidence()[0] = 'Y';
        signed.tag()[0] = 3;
        assertArrayEquals(originalEvidence, signed.evidence(),
                "mutating the evidence accessor result does not change the value");
        assertArrayEquals(originalTag, signed.tag(),
                "mutating the tag accessor result does not change the value");
    }

    @Test
    void entryCopiesInAndClonesOut() {
        byte[] payload = bytes("event-0");
        byte[] original = payload.clone();

        Entry entry = new Entry(new Event(0, OptionalLong.empty(), Hashes.sha256(payload), 0), payload);
        payload[0] = 'X';
        assertArrayEquals(original, entry.payload(),
                "caller mutation after construction does not change the entry");

        entry.payload()[0] = 'Y';
        assertArrayEquals(original, entry.payload(),
                "mutating an accessor result does not change the entry");
    }

    @Test
    void auditLogIsNotMutableThroughTheCallerPayload() {
        ChainedAudit audit = new ChainedAudit();
        byte[] payload = bytes("event-0");
        audit.append(payload, 0);

        payload[0] = 'X';
        EvidenceBundle bundle = audit.exportRange(0, 0).orElseThrow();
        assertArrayEquals(bytes("event-0"), bundle.entries().get(0).payload(),
                "the log keeps its own copy of the payload");
        assertTrue(bundle.verify(), "the stored copy still matches its commitment");
    }
}
