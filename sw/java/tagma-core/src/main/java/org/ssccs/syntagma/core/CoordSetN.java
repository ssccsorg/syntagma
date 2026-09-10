package org.ssccs.syntagma.core;

import java.util.List;
import java.util.Objects;

/**
 * A sparse set of coordinates of depth {@code N}, backed by the
 * {@link CoordSpaceN} tree.
 *
 * <p>Memory allocates lazily: only paths that are actually inserted consume
 * nodes. Set operations walk the tree, O(entries) per walk. For depth 1 the
 * dense {@link CoordSet} is the better choice; the C++ and Rust references
 * document the same preference.
 *
 * <p>Java has no const generics, so the C++/Rust depth tag becomes immutable
 * instance state validated by the constructor.
 *
 * <p>Set operations reject a set of a different depth with
 * {@link IllegalArgumentException}; the C++ and Rust type systems make that
 * argument unrepresentable at compile time.
 *
 * <p>Port of the C++ {@code tagma::CoordSetN<N>} in
 * {@code sw/cpp/tagma_core/include/tagma_core/coord_set_n.h}; the underlying
 * behavior mirrors the Rust {@code CoordSetN<N>} in
 * {@code sw/rust/core/src/coord_set_n.rs}.
 */
public final class CoordSetN {

    /** The unit value stored in the backing space, the Java counterpart of {@code std::monostate}. */
    private static final Object UNIT = new Object();

    private final int depth;
    private final CoordSpaceN<Object> space;

    /**
     * Creates an empty set of {@code depth} levels.
     *
     * @throws IllegalArgumentException when {@code depth} is less than 1
     */
    public CoordSetN(int depth) {
        this.depth = depth;
        this.space = new CoordSpaceN<>(depth);
    }

    private CoordSetN(int depth, CoordSpaceN<Object> space) {
        this.depth = depth;
        this.space = space;
    }

    /** The number of tree levels, the C++/Rust {@code N} template argument. */
    public int depth() {
        return depth;
    }

    /** The number of coordinates in the set. */
    public int size() {
        return space.size();
    }

    /** Whether the set holds no coordinates. */
    public boolean isEmpty() {
        return space.isEmpty();
    }

    /**
     * Whether {@code path} is in the set.
     *
     * @throws IllegalArgumentException when {@code path.length() != depth()}
     */
    public boolean contains(CoordPath path) {
        return space.atPath(path).isPresent();
    }

    /**
     * Inserts {@code path}.
     *
     * @return {@code true} when {@code path} was newly inserted
     * @throws IllegalArgumentException when {@code path.length() != depth()}
     */
    public boolean insert(CoordPath path) {
        return space.placePath(path, UNIT).isEmpty();
    }

    /**
     * Removes {@code path}.
     *
     * @return {@code true} when {@code path} was present
     * @throws IllegalArgumentException when {@code path.length() != depth()}
     */
    public boolean remove(CoordPath path) {
        return space.vacatePath(path).isPresent();
    }

    /** Removes all coordinates from the set. */
    public void clear() {
        space.clear();
    }

    /** All paths in depth-first coordinate-ascending order. */
    public List<CoordPath> paths() {
        return space.paths();
    }

    /**
     * Returns an independent copy of the set, mirroring the Rust
     * {@code Clone} derive.
     */
    public CoordSetN copy() {
        return new CoordSetN(depth, space.copy());
    }

    // ------------------------------------------------------------------
    // Set operations
    // ------------------------------------------------------------------

    /**
     * The union of this set and {@code other}.
     *
     * @throws IllegalArgumentException when the sets have different depths
     */
    public CoordSetN union(CoordSetN other) {
        requireSameDepth(other);
        CoordSetN result = new CoordSetN(depth);
        for (CoordPath path : paths()) {
            result.insert(path);
        }
        for (CoordPath path : other.paths()) {
            result.insert(path);
        }
        return result;
    }

    /**
     * The intersection of this set and {@code other}. Iterates the smaller set
     * for efficiency, mirroring the references.
     *
     * @throws IllegalArgumentException when the sets have different depths
     */
    public CoordSetN intersection(CoordSetN other) {
        requireSameDepth(other);
        CoordSetN smaller = this;
        CoordSetN larger = other;
        if (size() > other.size()) {
            smaller = other;
            larger = this;
        }
        CoordSetN result = new CoordSetN(depth);
        for (CoordPath path : smaller.paths()) {
            if (larger.contains(path)) {
                result.insert(path);
            }
        }
        return result;
    }

    /**
     * The difference {@code this - other}: coordinates in this set that are not
     * in {@code other}.
     *
     * @throws IllegalArgumentException when the sets have different depths
     */
    public CoordSetN difference(CoordSetN other) {
        requireSameDepth(other);
        CoordSetN result = new CoordSetN(depth);
        for (CoordPath path : paths()) {
            if (!other.contains(path)) {
                result.insert(path);
            }
        }
        return result;
    }

    /**
     * The symmetric difference: coordinates in exactly one of the two sets.
     *
     * @throws IllegalArgumentException when the sets have different depths
     */
    public CoordSetN symmetricDifference(CoordSetN other) {
        requireSameDepth(other);
        CoordSetN result = new CoordSetN(depth);
        for (CoordPath path : paths()) {
            if (!other.contains(path)) {
                result.insert(path);
            }
        }
        for (CoordPath path : other.paths()) {
            if (!contains(path)) {
                result.insert(path);
            }
        }
        return result;
    }

    /**
     * Whether every path of this set is in {@code other}.
     *
     * @throws IllegalArgumentException when the sets have different depths
     */
    public boolean isSubset(CoordSetN other) {
        requireSameDepth(other);
        for (CoordPath path : paths()) {
            if (!other.contains(path)) {
                return false;
            }
        }
        return true;
    }

    /**
     * Whether this set contains every path of {@code other}.
     *
     * @throws IllegalArgumentException when the sets have different depths
     */
    public boolean isSuperset(CoordSetN other) {
        return other.isSubset(this);
    }

    /**
     * Whether the sets share no path. Iterates the smaller set for efficiency,
     * mirroring the references.
     *
     * @throws IllegalArgumentException when the sets have different depths
     */
    public boolean isDisjoint(CoordSetN other) {
        requireSameDepth(other);
        CoordSetN smaller = this;
        CoordSetN larger = other;
        if (size() > other.size()) {
            smaller = other;
            larger = this;
        }
        for (CoordPath path : smaller.paths()) {
            if (larger.contains(path)) {
                return false;
            }
        }
        return true;
    }

    // ------------------------------------------------------------------
    // Value semantics
    // ------------------------------------------------------------------

    /**
     * Content equality, mirroring the C++ {@code operator==} and the Rust
     * {@code Eq}: same depth, same length and mutual subset.
     */
    @Override
    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        if (!(other instanceof CoordSetN set)) {
            return false;
        }
        return depth == set.depth && size() == set.size() && isSubset(set);
    }

    /** Hash consistent with {@link #equals(Object)}; insertion order does not affect it. */
    @Override
    public int hashCode() {
        int hash = 31 * Integer.hashCode(depth) + Integer.hashCode(size());
        for (CoordPath path : paths()) {
            hash += path.hashCode();
        }
        return hash;
    }

    /** Displays the set, mirroring the Rust {@code Debug} impl. */
    @Override
    public String toString() {
        return "CoordSetN { depth: " + depth + ", size: " + size() + " }";
    }

    private void requireSameDepth(CoordSetN other) {
        Objects.requireNonNull(other, "other");
        if (depth != other.depth) {
            throw new IllegalArgumentException(
                    "CoordSetN depth mismatch: " + depth + " against " + other.depth);
        }
    }
}
