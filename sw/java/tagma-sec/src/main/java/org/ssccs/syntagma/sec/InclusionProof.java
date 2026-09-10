package org.ssccs.syntagma.sec;

import java.util.List;
import java.util.Objects;

/**
 * Chain segment from genesis through a target entry, verifiable without the
 * log.
 *
 * <p>{@code verify} accepts only a non-empty segment whose first entry is
 * genesis (no prev link), whose ids are exactly the positions {@code 0..=n},
 * and whose payload commitments and prev links are internally consistent.
 *
 * <p>Port of the C++ {@code tagma_sec::InclusionProof} in
 * {@code sw/cpp/tagma_sec/include/tagma_sec/audit.h}; the underlying behavior
 * mirrors the Rust {@code InclusionProof} in {@code sw/rust/sec/src/audit.rs}.
 */
public final class InclusionProof {

    private final List<Entry> entries;

    /**
     * Creates a proof over {@code entries}.
     *
     * @throws NullPointerException when the list or any entry is null
     */
    public InclusionProof(List<Entry> entries) {
        Objects.requireNonNull(entries, "entries");
        this.entries = List.copyOf(entries);
    }

    /** The entries of the proof as an unmodifiable list. */
    public List<Entry> entries() {
        return entries;
    }

    /**
     * Verifies ids {@code 0..=n}, prev links, and payload commitments.
     */
    public boolean verify() {
        if (entries.isEmpty()) {
            return false;
        }
        if (entries.get(0).event().prev().isPresent()) {
            return false;
        }
        for (int i = 0; i < entries.size(); i++) {
            if (entries.get(i).event().id() != i) {
                return false;
            }
        }
        return EntryChain.verify(entries);
    }
}
