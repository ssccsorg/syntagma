package org.ssccs.syntagma.sec;

import java.util.List;
import java.util.Objects;

/**
 * Exportable evidence bundle over a range of entries.
 *
 * <p>{@code verify} checks internal chain consistency and payload
 * commitments; the first entry may reference a predecessor outside the
 * bundle.
 *
 * <p>Port of the C++ {@code tagma_sec::EvidenceBundle} in
 * {@code sw/cpp/tagma_sec/include/tagma_sec/audit.h}; the underlying behavior
 * mirrors the Rust {@code EvidenceBundle} in {@code sw/rust/sec/src/audit.rs}.
 */
public final class EvidenceBundle {

    private final List<Entry> entries;

    /**
     * Creates a bundle over {@code entries}.
     *
     * @throws NullPointerException when the list or any entry is null
     */
    public EvidenceBundle(List<Entry> entries) {
        Objects.requireNonNull(entries, "entries");
        this.entries = List.copyOf(entries);
    }

    /** The entries of the bundle as an unmodifiable list. */
    public List<Entry> entries() {
        return entries;
    }

    /**
     * Verifies internal chain consistency and payload commitments.
     */
    public boolean verify() {
        return EntryChain.verify(entries);
    }
}
