package org.ssccs.syntagma.sec;

/**
 * Channel: non-repudiation of origin and receipt.
 *
 * <p>Evidence is signed with a keyed tag and verified by recomputation. A
 * production deployment replaces this with a signature scheme over the module
 * interface.
 *
 * <p>Port of the C++ {@code tagma_sec::Channel} in
 * {@code sw/cpp/tagma_sec/include/tagma_sec/channel.h}; the underlying
 * behavior mirrors the Rust {@code Channel} trait in
 * {@code sw/rust/sec/src/channel.rs}.
 */
public interface Channel {

    /**
     * Signs evidence, producing a verifiable signed message.
     */
    SignedEvidence sign(byte[] evidence);

    /**
     * Accepts only signed evidence whose tag matches the evidence.
     */
    boolean verify(SignedEvidence signedEvidence);

    /**
     * Produces a receipt binding {@code local}, {@code remote}, and
     * {@code epoch}.
     */
    Receipt exchange(byte[] local, long remote, long epoch);

    /**
     * Accepts only receipts whose signed evidence matches the remote and
     * epoch recorded on the receipt.
     */
    boolean verifyReceipt(Receipt receipt);
}
