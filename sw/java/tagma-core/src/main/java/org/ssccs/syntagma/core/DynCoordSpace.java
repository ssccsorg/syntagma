package org.ssccs.syntagma.core;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Optional;

/**
 * A collision-free space indexed by a slice of coordinates with dynamic depth.
 *
 * <p>Each level is a fixed 11,172-slot array indexed directly by {@link Coord},
 * so navigation costs one array access per coordinate, with no hashing and no
 * collisions regardless of depth. The depth is determined at runtime by the
 * length of the path; memory allocates lazily, and only paths that are actually
 * written consume nodes.
 *
 * <p>A path index can hold a value and a deeper subtree at the same time (the
 * C++ {@code kBoth} slot, the Rust {@code Slot::Both}), so a placement never
 * displaces deeper paths and a removal of a prefix value preserves the paths
 * below it.
 *
 * <p>Port of the C++ {@code tagma::DynCoordSpace<V>} in
 * {@code sw/cpp/tagma_core/include/tagma_core/dyn_coord_space.h}; the underlying
 * behavior mirrors the Rust {@code DynCoordSpace<V>} in
 * {@code sw/rust/core/src/dyn_coord_space.rs}.
 *
 * @param <V> the stored value type
 */
public final class DynCoordSpace<V> {

    /** The number of slots per level, always 11,172. */
    public static final int CAPACITY = Coord.N_VALID;

    /** One slot per valid coordinate at this level; null is the empty slot. */
    private final Object[] slots = new Object[CAPACITY];

    /** Creates an empty space. */
    public DynCoordSpace() {
    }

    /**
     * The value at {@code path}.
     *
     * @return empty when the path is empty or absent
     */
    public Optional<V> at(List<Coord> path) {
        Objects.requireNonNull(path, "path");
        if (path.isEmpty()) {
            return Optional.empty();
        }
        DynCoordSpace<V> node = this;
        for (int i = 0; i < path.size(); i++) {
            Coord coord = Objects.requireNonNull(path.get(i), "null coordinate in path");
            Slot<V> slot = node.slotAt(coord.index());
            if (slot == null) {
                return Optional.empty();
            }
            boolean last = i == path.size() - 1;
            switch (slot.kind) {
                case LEAF, BOTH -> {
                    if (last) {
                        return Optional.of(slot.value);
                    }
                    if (slot.child == null) {
                        return Optional.empty();
                    }
                    node = slot.child;
                }
                case NODE -> {
                    if (last) {
                        // A node holds no value at its own level.
                        return Optional.empty();
                    }
                    node = slot.child;
                }
            }
        }
        return Optional.empty();
    }

    /**
     * Inserts {@code value} at {@code path}, creating intermediate nodes as
     * needed.
     *
     * @return the previous value when the exact path already existed
     * @throws IllegalArgumentException when {@code path} is empty, mirroring
     *         the Rust panic and the C++ {@code std::invalid_argument}
     * @throws NullPointerException when {@code value} is null
     */
    public Optional<V> place(List<Coord> path, V value) {
        Objects.requireNonNull(path, "path");
        Objects.requireNonNull(value, "null value");
        if (path.isEmpty()) {
            throw new IllegalArgumentException("DynCoordSpace: path must not be empty");
        }
        return insertRec(path, 0, value);
    }

    /**
     * Removes the value at {@code path}.
     *
     * @return the removed value when present; empty when the path is empty or
     *         absent
     */
    public Optional<V> vacate(List<Coord> path) {
        Objects.requireNonNull(path, "path");
        if (path.isEmpty()) {
            return Optional.empty();
        }
        return removeRec(path, 0);
    }

    /** Removes all entries. */
    public void clear() {
        Arrays.fill(slots, null);
    }

    /** The number of entries across all depths, O(entries). */
    public int entryCount() {
        return countRec();
    }

    /**
     * All {@code (path, value)} pairs in depth-first coordinate-ascending
     * order, as a snapshot. The Rust reference yields a lazy iterator; the Java
     * port materializes it eagerly, as the C++ port does.
     */
    public List<Map.Entry<List<Coord>, V>> entries() {
        List<Map.Entry<List<Coord>, V>> out = new ArrayList<>();
        collect(new ArrayList<>(), out);
        return out;
    }

    /**
     * Returns an independent copy of the tree, mirroring the Rust
     * {@code Clone} derive that the C++ port omits. Values are shared, not
     * cloned: Java has no generic clone contract, and the port treats stored
     * values as immutable.
     */
    public DynCoordSpace<V> copy() {
        DynCoordSpace<V> copy = new DynCoordSpace<>();
        for (int index = 0; index < CAPACITY; index++) {
            Slot<V> slot = slotAt(index);
            if (slot != null) {
                Slot<V> clone = new Slot<>(slot.kind);
                clone.value = slot.value;
                clone.child = slot.child == null ? null : slot.child.copy();
                copy.slots[index] = clone;
            }
        }
        return copy;
    }

    // ------------------------------------------------------------------
    // Internals
    // ------------------------------------------------------------------

    /**
     * One slot, mirroring the C++ {@code Slot} and the Rust {@code Slot} enum:
     * a leaf holds a value, a node holds a child space, a both slot holds a
     * value and a child space simultaneously.
     */
    private static final class Slot<V> {

        /** The three slot roles, mirroring the C++ {@code Slot::Kind} and Rust {@code Slot}. */
        private enum Kind {
            LEAF,
            NODE,
            BOTH
        }

        private Kind kind;
        private V value;
        private DynCoordSpace<V> child;

        private Slot(Kind kind) {
            this.kind = kind;
        }
    }

    private Optional<V> insertRec(List<Coord> path, int depth, V value) {
        Coord coord = Objects.requireNonNull(path.get(depth), "null coordinate in path");
        int index = coord.index();
        Slot<V> slot = slotAt(index);
        if (depth == path.size() - 1) {
            if (slot == null) {
                slots[index] = new Slot<>(Slot.Kind.LEAF);
                slot = slotAt(index);
                slot.value = value;
                return Optional.empty();
            }
            switch (slot.kind) {
                case LEAF, BOTH -> {
                    V previous = slot.value;
                    slot.value = value;
                    return Optional.of(previous);
                }
                case NODE -> {
                    // The node held no value at this level; keep its subtree.
                    slot.kind = Slot.Kind.BOTH;
                    slot.value = value;
                    return Optional.empty();
                }
            }
        }
        if (slot == null) {
            slots[index] = slot = new Slot<>(Slot.Kind.NODE);
            slot.child = new DynCoordSpace<>();
        } else if (slot.kind == Slot.Kind.LEAF) {
            // Promote a leaf into a both slot, preserving the existing value.
            slot.kind = Slot.Kind.BOTH;
            slot.child = new DynCoordSpace<>();
        }
        return slot.child.insertRec(path, depth + 1, value);
    }

    private Optional<V> removeRec(List<Coord> path, int depth) {
        Coord coord = Objects.requireNonNull(path.get(depth), "null coordinate in path");
        int index = coord.index();
        Slot<V> slot = slotAt(index);
        if (slot == null) {
            return Optional.empty();
        }
        if (depth == path.size() - 1) {
            switch (slot.kind) {
                case LEAF -> {
                    slots[index] = null;
                    return Optional.of(slot.value);
                }
                case BOTH -> {
                    V value = slot.value;
                    slot.kind = Slot.Kind.NODE; // preserve deeper paths
                    slot.value = null;
                    return Optional.of(value);
                }
                case NODE -> {
                    return Optional.empty();
                }
            }
        }
        if (slot.kind == Slot.Kind.LEAF) {
            return Optional.empty();
        }
        return slot.child.removeRec(path, depth + 1);
    }

    private int countRec() {
        int count = 0;
        for (int index = 0; index < CAPACITY; index++) {
            Slot<V> slot = slotAt(index);
            if (slot == null) {
                continue;
            }
            switch (slot.kind) {
                case LEAF -> count += 1;
                case NODE -> count += slot.child.countRec();
                case BOTH -> count += 1 + slot.child.countRec();
            }
        }
        return count;
    }

    private void collect(List<Coord> path, List<Map.Entry<List<Coord>, V>> out) {
        for (int index = 0; index < CAPACITY; index++) {
            Slot<V> slot = slotAt(index);
            if (slot == null) {
                continue;
            }
            path.add(Coord.raw(index));
            switch (slot.kind) {
                case LEAF -> out.add(Map.entry(List.copyOf(path), slot.value));
                case NODE -> slot.child.collect(path, out);
                case BOTH -> {
                    out.add(Map.entry(List.copyOf(path), slot.value));
                    slot.child.collect(path, out);
                }
            }
            path.remove(path.size() - 1);
        }
    }

    /** Reads a slot. One suppression covers the class; Java generics are erased. */
    @SuppressWarnings("unchecked")
    private Slot<V> slotAt(int index) {
        return (Slot<V>) slots[index];
    }
}
