package org.ssccs.syntagma.sec;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.ssccs.syntagma.sec.Fixtures.bytes;

import java.util.Arrays;
import java.util.List;
import java.util.OptionalLong;
import org.junit.jupiter.api.Test;

/**
 * Rejection paths of the verification branches: inclusion proofs, evidence
 * bundles, and receipts.
 *
 * <p>The accepting cases are covered by the workflow and scenario suites;
 * these cases pin the negative branches the javadoc of
 * {@link InclusionProof#verify()}, {@link EntryChain}, and
 * {@link MacChannel#verifyReceipt} describes, so deleting the genesis check,
 * the id check, the prev-link check, or the truncated-evidence guard fails
 * here instead of passing silently. Mirrors the C++ suite
 * {@code sw/cpp/tagma_sec/tests/test_verification.cpp}.
 */
class VerificationTest {

    /**
     * An entry whose commitment matches its payload, with the id also used as
     * the epoch so every other check of the verification branches still passes.
     */
    private static Entry entry(long id, OptionalLong prev, String payload) {
        byte[] body = bytes(payload);
        return new Entry(new Event(id, prev, Hashes.sha256(body), id), body);
    }

    @Test
    void inclusionProofRejectsNonGenesisFirstEntry() {
        InclusionProof proof = new InclusionProof(List.of(entry(0, OptionalLong.of(0), "event-0")));
        assertFalse(proof.verify(), "a first entry carrying a prev link is rejected");
    }

    @Test
    void inclusionProofRejectsShiftedIds() {
        InclusionProof proof = new InclusionProof(List.of(
                entry(1, OptionalLong.empty(), "event-0"),
                entry(2, OptionalLong.of(1), "event-1")));
        assertFalse(proof.verify(), "ids that do not start at zero are rejected");
    }

    @Test
    void inclusionProofRejectsEmptySegment() {
        InclusionProof proof = new InclusionProof(List.of());
        assertFalse(proof.verify(), "an empty segment is rejected");
    }

    @Test
    void evidenceBundleRejectsBrokenPrevLinks() {
        EvidenceBundle unrelated = new EvidenceBundle(List.of(
                entry(0, OptionalLong.empty(), "event-0"),
                entry(1, OptionalLong.of(5), "event-1")));
        assertFalse(unrelated.verify(), "a prev link to an unrelated id is rejected");

        EvidenceBundle missing = new EvidenceBundle(List.of(
                entry(0, OptionalLong.empty(), "event-0"),
                entry(1, OptionalLong.empty(), "event-1")));
        assertFalse(missing.verify(), "a missing prev link after the first entry is rejected");
    }

    @Test
    void receiptEvidenceLengthBoundary() {
        MacChannel channel = new MacChannel();
        byte[] body = new byte[16];
        System.arraycopy(Hashes.le64(7), 0, body, 0, 8);
        System.arraycopy(Hashes.le64(5), 0, body, 8, 8);

        Receipt exact = new Receipt(channel.sign(body), 7, 5);
        assertTrue(channel.verifyReceipt(exact), "a 16-byte receipt body verifies");

        byte[] truncated = Arrays.copyOf(body, 15);
        Receipt shortEvidence = new Receipt(channel.sign(truncated), 7, 5);
        assertFalse(channel.verifyReceipt(shortEvidence),
                "a receipt with truncated evidence is rejected");
    }
}
