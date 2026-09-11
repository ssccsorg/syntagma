package org.ssccs.syntagma.sec;

import java.util.ArrayList;
import java.util.List;
import java.util.Objects;
import java.util.Optional;
import java.util.OptionalLong;

/**
 * Chained append-only log shared by the legacy and tagma-sec stacks.
 *
 * <p>The log assigns ids by position, records the predecessor id in every
 * entry after genesis, and commits each payload with SHA-256. Every returned
 * entry and bundle holds defensive copies, so a caller cannot mutate the log
 * through the values it receives.
 *
 * <p>Port of the C++ {@code tagma_sec::ChainedAudit} in
 * {@code sw/cpp/tagma_sec/include/tagma_sec/audit.h}; the underlying behavior
 * mirrors the Rust {@code ChainedAudit} in {@code sw/rust/sec/src/audit.rs}.
 */
public final class ChainedAudit implements Audit {

    private final List<Event> events = new ArrayList<>();
    private final List<byte[]> payloads = new ArrayList<>();

    @Override
    public Event append(byte[] payload, long epoch) {
        Objects.requireNonNull(payload, "payload");
        OptionalLong prev = events.isEmpty()
                ? OptionalLong.empty()
                : OptionalLong.of(events.get(events.size() - 1).id());
        Event event = new Event(events.size(), prev, Hashes.sha256(payload), epoch);
        events.add(event);
        payloads.add(payload.clone());
        return event;
    }

    @Override
    public boolean verifyChain(long from, long to) {
        if (from < 0 || from > to || to >= events.size()) {
            return false;
        }
        for (long i = from; i <= to; i++) {
            int index = (int) i;
            if (i == 0) {
                if (events.get(index).prev().isPresent()) {
                    return false;
                }
            } else if (events.get(index).prev().isEmpty()
                    || events.get(index).prev().getAsLong() != events.get(index - 1).id()) {
                return false;
            }
        }
        return true;
    }

    @Override
    public Optional<InclusionProof> prove(long entry) {
        if (entry < 0 || entry >= events.size()) {
            return Optional.empty();
        }
        List<Entry> segment = new ArrayList<>();
        for (int i = 0; i <= entry; i++) {
            segment.add(new Entry(events.get(i), payloads.get(i)));
        }
        return Optional.of(new InclusionProof(segment));
    }

    @Override
    public Optional<EvidenceBundle> exportRange(long from, long to) {
        if (from < 0 || from > to || to >= events.size()) {
            return Optional.empty();
        }
        List<Entry> range = new ArrayList<>();
        for (long i = from; i <= to; i++) {
            int index = (int) i;
            range.add(new Entry(events.get(index), payloads.get(index)));
        }
        return Optional.of(new EvidenceBundle(range));
    }
}
