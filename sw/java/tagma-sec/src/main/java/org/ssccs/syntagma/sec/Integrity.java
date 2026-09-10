package org.ssccs.syntagma.sec;

import java.util.Optional;
import org.ssccs.syntagma.core.CoordPath;

/**
 * Integrity: tamper-evident binding of a record to its path, principal, and
 * epoch.
 *
 * <p>The seal is a keyed commitment. It does not hide the record; it makes
 * modification evident. Implementations differ in whether the epoch and the
 * principal participate in the binding: the legacy pattern ignores them.
 *
 * <p>Port of the C++ {@code tagma_sec::Integrity} in
 * {@code sw/cpp/tagma_sec/include/tagma_sec/integrity.h}; the underlying
 * behavior mirrors the Rust {@code Integrity} trait in
 * {@code sw/rust/sec/src/integrity.rs}.
 */
public interface Integrity {

    /**
     * Binds {@code record} to {@code path}, {@code principal}, and
     * {@code epoch}, returning the keyed seal.
     */
    Seal seal(byte[] record, CoordPath path, long principal, long epoch);

    /**
     * Accepts only seals that match all bound inputs.
     */
    boolean verify(byte[] record, CoordPath path, long principal, long epoch, Seal seal);

    /**
     * Re-binds an existing seal to a newer epoch without altering the record.
     *
     * @return the seal bound to {@code toEpoch}, or empty when the input seal
     *         does not verify
     */
    Optional<Seal> refresh(byte[] record, CoordPath path, long principal, long fromEpoch,
            long toEpoch, Seal seal);
}
