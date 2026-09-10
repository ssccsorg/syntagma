package org.ssccs.syntagma.sec;

import java.util.Arrays;
import java.util.Objects;
import java.util.Optional;
import org.ssccs.syntagma.core.Coord;
import org.ssccs.syntagma.core.CoordPath;

/**
 * Integrity seal for the legacy pattern: binds the record and the path only.
 * The principal and the epoch do not participate, so a legacy seal stays valid
 * across epoch changes and refreshes re-emit the identical tag.
 *
 * <p>Port of the C++ {@code tagma_sec::LegacyIntegrity} in
 * {@code sw/cpp/tagma_sec/include/tagma_sec/legacy.h}; the underlying behavior
 * mirrors the Rust {@code LegacyIntegrity} in
 * {@code sw/rust/sec/src/legacy.rs}.
 */
public final class LegacyIntegrity implements Integrity {

    private static final byte[] LEGACY_KEY = Hashes.asciiKey("tagma-sec-poc-legacy-int-key-000");

    @Override
    public Seal seal(byte[] record, CoordPath path, long principal, long epoch) {
        Objects.requireNonNull(record, "record");
        Objects.requireNonNull(path, "path");
        Coord[] coords = path.coords();
        byte[][] parts = new byte[coords.length + 1][];
        parts[0] = record;
        for (int i = 0; i < coords.length; i++) {
            parts[i + 1] = Hashes.le16(coords[i].index());
        }
        return new Seal(Hashes.keyedTag(LEGACY_KEY, parts));
    }

    @Override
    public boolean verify(byte[] record, CoordPath path, long principal, long epoch, Seal seal) {
        Objects.requireNonNull(seal, "seal");
        return Arrays.equals(seal(record, path, 0, 0).tag(), seal.tag());
    }

    @Override
    public Optional<Seal> refresh(byte[] record, CoordPath path, long principal, long fromEpoch,
            long toEpoch, Seal seal) {
        if (!verify(record, path, principal, 0, seal)) {
            return Optional.empty();
        }
        return Optional.of(seal(record, path, principal, toEpoch));
    }
}
