package org.ssccs.syntagma.sec;

import java.util.List;
import java.util.Objects;
import org.ssccs.syntagma.core.Coord;
import org.ssccs.syntagma.core.CoordPath;

/**
 * A scope names a target path or a set of paths.
 *
 * <p>Matching follows the Milestone 1 contract: Exact matches when the target
 * path equals the scope path, Prefix matches when the scope path is a prefix
 * of the target path. An empty prefix scope is the root scope and covers every
 * path.
 *
 * <p>Port of the C++ {@code tagma_sec::Scope} in
 * {@code sw/cpp/tagma_sec/include/tagma_sec/types.h}; the underlying behavior
 * mirrors the Rust {@code Scope} in {@code sw/rust/sec/src/types.rs}. The C++
 * {@code std::vector<Coord>} path maps to the core
 * {@link org.ssccs.syntagma.core.CoordPath}.
 */
public final class Scope {

    private enum Kind {
        EXACT,
        PREFIX
    }

    private final Kind kind;
    private final CoordPath path;

    private Scope(Kind kind, CoordPath path) {
        this.kind = kind;
        this.path = path;
    }

    /**
     * An exact scope: the target path must equal {@code path}.
     *
     * @throws NullPointerException when {@code path} is null
     */
    public static Scope exact(CoordPath path) {
        return new Scope(Kind.EXACT, Objects.requireNonNull(path, "path"));
    }

    /**
     * A prefix scope: {@code path} must be a prefix of the target path.
     *
     * @throws NullPointerException when {@code path} is null
     */
    public static Scope prefix(CoordPath path) {
        return new Scope(Kind.PREFIX, Objects.requireNonNull(path, "path"));
    }

    /**
     * Returns true when {@code target} falls inside this scope under the
     * Milestone 1 matching rules.
     */
    public boolean matches(CoordPath target) {
        Objects.requireNonNull(target, "target");
        if (kind == Kind.EXACT) {
            return path.equals(target);
        }
        Coord[] scope = path.coords();
        Coord[] candidate = target.coords();
        if (scope.length > candidate.length) {
            return false;
        }
        for (int i = 0; i < scope.length; i++) {
            if (!scope[i].equals(candidate[i])) {
                return false;
            }
        }
        return true;
    }

    /**
     * A stable key for revocation bookkeeping: a rule marker (0 for exact, 1
     * for prefix) followed by the coordinate indices of the scope path.
     *
     * <p>Mirrors the C++ {@code key}, whose {@code std::vector<uint16_t>}
     * becomes an immutable integer sequence so it can key the revocation map
     * directly.
     */
    public List<Integer> key() {
        Coord[] coords = path.coords();
        Integer[] key = new Integer[coords.length + 1];
        key[0] = kind == Kind.EXACT ? 0 : 1;
        for (int i = 0; i < coords.length; i++) {
            key[i + 1] = coords[i].index();
        }
        return List.of(key);
    }

    /** The scope path. */
    public CoordPath path() {
        return path;
    }

    @Override
    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        if (!(other instanceof Scope scope)) {
            return false;
        }
        return kind == scope.kind && path.equals(scope.path);
    }

    @Override
    public int hashCode() {
        return 31 * kind.hashCode() + path.hashCode();
    }
}
