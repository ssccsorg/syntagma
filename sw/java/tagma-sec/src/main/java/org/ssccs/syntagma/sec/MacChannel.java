package org.ssccs.syntagma.sec;

import java.util.Arrays;
import java.util.Objects;

/**
 * Keyed-tag channel shared by the legacy and tagma-sec stacks.
 *
 * <p>{@code exchange} appends the little-endian remote principal and epoch to
 * the local evidence, signs the result, and records both on the receipt;
 * {@code verifyReceipt} decodes the tail of the signed evidence and checks it
 * against the receipt fields before verifying the tag, so tampering with
 * either the evidence or the recorded remote and epoch is detected.
 *
 * <p>Port of the C++ {@code tagma_sec::MacChannel} in
 * {@code sw/cpp/tagma_sec/include/tagma_sec/channel.h}; the underlying
 * behavior mirrors the Rust {@code MacChannel} in
 * {@code sw/rust/sec/src/channel.rs}.
 */
public final class MacChannel implements Channel {

    private static final byte[] CHANNEL_KEY = Hashes.asciiKey("tagma-sec-poc-channel-key-000000");

    @Override
    public SignedEvidence sign(byte[] evidence) {
        Objects.requireNonNull(evidence, "evidence");
        return new SignedEvidence(evidence, Hashes.keyedTag(CHANNEL_KEY, evidence));
    }

    @Override
    public boolean verify(SignedEvidence signedEvidence) {
        Objects.requireNonNull(signedEvidence, "signedEvidence");
        byte[] expected = Hashes.keyedTag(CHANNEL_KEY, signedEvidence.evidence());
        return Arrays.equals(expected, signedEvidence.tag());
    }

    @Override
    public Receipt exchange(byte[] local, long remote, long epoch) {
        Objects.requireNonNull(local, "local");
        byte[] evidence = new byte[local.length + 16];
        System.arraycopy(local, 0, evidence, 0, local.length);
        System.arraycopy(Hashes.le64(remote), 0, evidence, local.length, 8);
        System.arraycopy(Hashes.le64(epoch), 0, evidence, local.length + 8, 8);
        return new Receipt(sign(evidence), remote, epoch);
    }

    @Override
    public boolean verifyReceipt(Receipt receipt) {
        Objects.requireNonNull(receipt, "receipt");
        byte[] evidence = receipt.signedEvidence().evidence();
        if (evidence.length < 16) {
            return false;
        }
        long remote = readLe64(evidence, evidence.length - 16);
        long epoch = readLe64(evidence, evidence.length - 8);
        return verify(receipt.signedEvidence())
                && remote == receipt.remote()
                && epoch == receipt.epoch();
    }

    /** Reads the little-endian 64-bit value at {@code offset}. */
    private static long readLe64(byte[] data, int offset) {
        long value = 0;
        for (int i = 0; i < 8; i++) {
            value |= (long) (data[offset + i] & 0xFF) << (8 * i);
        }
        return value;
    }
}
