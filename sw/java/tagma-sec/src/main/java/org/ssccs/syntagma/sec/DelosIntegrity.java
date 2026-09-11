package org.ssccs.syntagma.sec;

import java.security.MessageDigest;
import java.util.Objects;
import java.util.Optional;
import org.ssccs.syntagma.core.Coord;
import org.ssccs.syntagma.core.CoordPath;

/**
 * Integrity seal for the tagma-sec pattern: binds the record, the path, the
 * principal, and the epoch, so replay across epochs is detected.
 *
 * <p>The tagged message is the record, the little-endian principal, the
 * little-endian epoch, and the little-endian coordinate indices, in that
 * order. {@code refresh} rejects a target epoch that is not newer than the
 * binding epoch.
 *
 * <p>Tag comparison goes through {@link MessageDigest#isEqual}, which inspects
 * every byte without an early exit, so a mismatch does not reveal where the
 * difference lies. {@code Arrays.equals} is avoided in verification paths for
 * that reason.
 *
 * <p>Port of the C++ {@code tagma_sec::DelosIntegrity} in
 * {@code sw/cpp/tagma_sec/include/tagma_sec/delos.h}; the underlying behavior
 * mirrors the Rust {@code DelosIntegrity} in {@code sw/rust/sec/src/delos.rs}.
 */
public final class DelosIntegrity implements Integrity {

    private static final byte[] DELOS_KEY = Hashes.asciiKey("tagma-sec-poc-delos-int-key-0000");

    @Override
    public Seal seal(byte[] record, CoordPath path, long principal, long epoch) {
        Objects.requireNonNull(record, "record");
        Objects.requireNonNull(path, "path");
        Coord[] coords = path.coords();
        byte[][] parts = new byte[coords.length + 3][];
        parts[0] = record;
        parts[1] = Hashes.le64(principal);
        parts[2] = Hashes.le64(epoch);
        for (int i = 0; i < coords.length; i++) {
            parts[i + 3] = Hashes.le16(coords[i].index());
        }
        return new Seal(Hashes.keyedTag(DELOS_KEY, parts));
    }

    @Override
    public boolean verify(byte[] record, CoordPath path, long principal, long epoch, Seal seal) {
        Objects.requireNonNull(seal, "seal");
        return MessageDigest.isEqual(seal(record, path, principal, epoch).tag(), seal.tag());
    }

    @Override
    public Optional<Seal> refresh(byte[] record, CoordPath path, long principal, long fromEpoch,
            long toEpoch, Seal seal) {
        if (toEpoch <= fromEpoch) {
            return Optional.empty();
        }
        if (!verify(record, path, principal, fromEpoch, seal)) {
            return Optional.empty();
        }
        return Optional.of(seal(record, path, principal, toEpoch));
    }
}
