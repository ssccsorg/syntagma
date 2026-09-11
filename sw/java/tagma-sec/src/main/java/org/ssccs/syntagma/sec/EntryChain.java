package org.ssccs.syntagma.sec;

import java.security.MessageDigest;
import java.util.List;
import java.util.OptionalLong;

/**
 * Shared chain verification for {@link InclusionProof} and
 * {@link EvidenceBundle}: payload commitments and prev links.
 *
 * <p>The first entry may carry a prev link to an entry outside the bundle, so
 * internal consistency is checked from the second entry onward. Mirrors the
 * static {@code verify_entries} of the C++ {@code audit.cpp} and the Rust
 * {@code verify_entries} in {@code sw/rust/sec/src/audit.rs}.
 *
 * <p>Package-private: the shared helper is not part of the module interface.
 * Commitments are compared through {@link MessageDigest#isEqual}, which
 * inspects every byte without an early exit.
 */
final class EntryChain {

    private EntryChain() {
    }

    static boolean verify(List<Entry> entries) {
        for (int i = 0; i < entries.size(); i++) {
            Entry entry = entries.get(i);
            if (!MessageDigest.isEqual(Hashes.sha256(entry.payload()), entry.event().payloadHash())) {
                return false;
            }
            if (i > 0) {
                OptionalLong prev = entry.event().prev();
                if (prev.isEmpty()) {
                    return false;
                }
                if (prev.getAsLong() != entries.get(i - 1).event().id()) {
                    return false;
                }
            }
        }
        return true;
    }
}
