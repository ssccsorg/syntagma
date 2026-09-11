package org.ssccs.syntagma.sec;

import java.util.Objects;
import java.util.Optional;
import org.ssccs.syntagma.core.Coord;
import org.ssccs.syntagma.core.CoordPath;

/**
 * A composable security stack with switchable implementations.
 *
 * <p>{@link #legacy()} composes the legacy pattern and {@link #delos()} the
 * tagma-sec pattern; the scenario workflow is written only against the module
 * interfaces, so the same requirements and outputs are exercised against both
 * implementations.
 *
 * <p>Port of the C++ {@code tagma_sec::SecStack} and the {@code route_update}
 * entry point in {@code sw/cpp/tagma_sec/include/tagma_sec/proxy.h}; the
 * underlying behavior mirrors the Rust {@code SecStack} and {@code
 * route_update} in {@code sw/rust/sec/src/proxy.rs}.
 */
public record SecStack(Authority authority, Integrity integrity, Audit audit, Channel channel) {

    /**
     * Compact canonical constructor: every module must be provided.
     */
    public SecStack {
        Objects.requireNonNull(authority, "authority");
        Objects.requireNonNull(integrity, "integrity");
        Objects.requireNonNull(audit, "audit");
        Objects.requireNonNull(channel, "channel");
    }

    /**
     * The legacy security pattern stack.
     */
    public static SecStack legacy() {
        return new SecStack(
                new LegacyAuthority(),
                new LegacyIntegrity(),
                new ChainedAudit(),
                new MacChannel());
    }

    /**
     * The tagma-sec pattern stack.
     */
    public static SecStack delos() {
        return new SecStack(
                new DelosAuthority(),
                new DelosIntegrity(),
                new ChainedAudit(),
                new MacChannel());
    }

    /**
     * Scenario workflow: a coordination node submits a route update.
     *
     * <p>The update is accepted only when the attestation authorizes the path
     * and action at the current epoch and the sealed record verifies. The
     * record is then committed to the audit log, and the channel exchanges a
     * receipt binding the record to its path, epoch, and origin principal;
     * the signed evidence is committed to the audit log as the
     * non-repudiation receipt.
     *
     * @return the audit entry and receipt of an accepted update, or empty on
     *         any rejected step
     */
    public static Optional<RouteUpdate> routeUpdate(SecStack stack, Attestation att, CoordPath path,
            byte[] record, long epoch) {
        Objects.requireNonNull(stack, "stack");
        Objects.requireNonNull(att, "att");
        Objects.requireNonNull(path, "path");
        Objects.requireNonNull(record, "record");
        if (stack.authority().authorize(att, path, Authority.ACTION_ROUTE_UPDATE, epoch)
                != Decision.ALLOW) {
            return Optional.empty();
        }
        Seal seal = stack.integrity().seal(record, path, att.principal(), epoch);
        if (!stack.integrity().verify(record, path, att.principal(), epoch, seal)) {
            return Optional.empty();
        }
        Event event = stack.audit().append(record, epoch);

        // Non-repudiation: bind the record to its path and epoch, exchange a
        // receipt with the channel, and commit the signed evidence to the
        // audit log (receipts are appended by the caller per
        // docs/spec/tagma-sec.md).
        Coord[] coords = path.coords();
        byte[] evidence = new byte[record.length + coords.length * 2 + 8];
        int offset = 0;
        System.arraycopy(record, 0, evidence, offset, record.length);
        offset += record.length;
        for (Coord coord : coords) {
            System.arraycopy(Hashes.le16(coord.index()), 0, evidence, offset, 2);
            offset += 2;
        }
        System.arraycopy(Hashes.le64(epoch), 0, evidence, offset, 8);

        Receipt receipt = stack.channel().exchange(evidence, att.principal(), epoch);
        stack.audit().append(receipt.signedEvidence().evidence(), epoch);

        return Optional.of(new RouteUpdate(event, receipt));
    }
}
