package org.ssccs.syntagma.core;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.NoSuchElementException;
import java.util.Objects;
import java.util.Optional;
import java.util.function.BiPredicate;
import java.util.function.Supplier;
import java.util.function.UnaryOperator;

/**
 * A hash-less, collision-free, single-character direct-address table.
 *
 * <p>One slot per valid coordinate (11,172), so placement and lookup are O(1)
 * with a single array access and no hashing. The C++ reference keeps the slot
 * array inline; Java stores it on the heap, which has the same observable
 * semantics.
 *
 * <p>Null values are not permitted: a vacant slot and an absent value are the
 * same state, mirroring {@code Option<V>} / {@code std::optional<V>} in the
 * Rust and C++ references.
 *
 * <p>Port of the C++ {@code tagma::CoordSpace<V>} in
 * {@code sw/cpp/tagma_core}; the underlying behavior mirrors the Rust
 * {@code CoordSpace<V>} in {@code sw/rust/core}.
 *
 * @param <V> the stored value type
 */
public final class CoordSpace<V> {

    /** The maximum capacity, always 11,172. */
    public static final int CAPACITY = Coord.N_VALID;

    private final Object[] slots = new Object[CAPACITY];
    private int size;

    /** Creates an empty {@code CoordSpace}. */
    public CoordSpace() {
    }

    // ------------------------------------------------------------------
    // Read
    // ------------------------------------------------------------------

    /** The number of entries. */
    public int size() {
        return size;
    }

    /** Whether the space holds no entries. */
    public boolean isEmpty() {
        return size == 0;
    }

    /** The maximum capacity, always 11,172. */
    public static int capacity() {
        return CAPACITY;
    }

    /**
     * The value at {@code coord}.
     *
     * @return empty when the slot is vacant
     */
    @SuppressWarnings("unchecked")
    public Optional<V> at(Coord coord) {
        return Optional.ofNullable((V) slots[coord.index()]);
    }

    /**
     * A mutable handle to the value at {@code coord}.
     *
     * @return empty when the slot is vacant, mirroring the C++ {@code at_mut}
     *         pointer return
     */
    public Optional<ValueRef<V>> atMut(Coord coord) {
        return occupied(coord) ? Optional.of(new ValueRef<>(this, coord)) : Optional.empty();
    }

    /** Whether the space holds an entry for {@code coord}. */
    public boolean occupied(Coord coord) {
        return slots[coord.index()] != null;
    }

    /** The value at the single-coordinate path. */
    public Optional<V> atPath(CoordPath path) {
        return at(single(path));
    }

    // ------------------------------------------------------------------
    // Write
    // ------------------------------------------------------------------

    /**
     * Inserts {@code value} at {@code coord}.
     *
     * @return the previous value when the slot was occupied
     * @throws NullPointerException when {@code value} is null
     */
    public Optional<V> place(Coord coord, V value) {
        Objects.requireNonNull(value, "null value");
        int index = coord.index();
        @SuppressWarnings("unchecked")
        V previous = (V) slots[index];
        slots[index] = value;
        if (previous == null) {
            size += 1;
        }
        return Optional.ofNullable(previous);
    }

    /**
     * Removes and returns the value at {@code coord}.
     *
     * @return empty when the slot was vacant
     */
    public Optional<V> vacate(Coord coord) {
        int index = coord.index();
        @SuppressWarnings("unchecked")
        V previous = (V) slots[index];
        slots[index] = null;
        if (previous != null) {
            size -= 1;
        }
        return Optional.ofNullable(previous);
    }

    /** Removes all entries. */
    public void clear() {
        java.util.Arrays.fill(slots, null);
        size = 0;
    }

    /** Inserts a value at the single-coordinate path. */
    public Optional<V> placePath(CoordPath path, V value) {
        return place(single(path), value);
    }

    /** Removes and returns the value at the single-coordinate path. */
    public Optional<V> vacatePath(CoordPath path) {
        return vacate(single(path));
    }

    /**
     * Retains only the entries satisfying {@code keep}; removed entries are
     * dropped.
     */
    public void retain(BiPredicate<Coord, V> keep) {
        for (int index = 0; index < CAPACITY; index++) {
            if (slots[index] != null) {
                Coord coord = Coord.raw(index);
                @SuppressWarnings("unchecked")
                V value = (V) slots[index];
                if (!keep.test(coord, value)) {
                    slots[index] = null;
                    size -= 1;
                }
            }
        }
    }

    // ------------------------------------------------------------------
    // Entry API
    // ------------------------------------------------------------------

    /**
     * Returns an entry handle for {@code coord}, mirroring the C++/Rust entry
     * API: {@code orInsert} inserts when the slot is vacant and never
     * overwrites an existing value.
     */
    public Entry<V> entry(Coord coord) {
        return new Entry<>(this, coord);
    }

    // ------------------------------------------------------------------
    // Iteration
    // ------------------------------------------------------------------

    /**
     * The occupied {@code (coord, value)} pairs in index order, as a snapshot.
     */
    public List<Map.Entry<Coord, V>> entries() {
        List<Map.Entry<Coord, V>> result = new ArrayList<>(size);
        for (int index = 0; index < CAPACITY; index++) {
            if (slots[index] != null) {
                @SuppressWarnings("unchecked")
                V value = (V) slots[index];
                result.add(Map.entry(Coord.raw(index), value));
            }
        }
        return result;
    }

    private static Coord single(CoordPath path) {
        Objects.requireNonNull(path, "path");
        if (path.length() != 1) {
            throw new IllegalArgumentException(
                    "CoordSpace path access requires a single-coordinate CoordPath, got length " + path.length());
        }
        return path.get(0).orElseThrow();
    }

    // ------------------------------------------------------------------
    // Nested handles
    // ------------------------------------------------------------------

    /**
     * A mutable reference to a value inside a {@link CoordSpace}, the Java
     * counterpart of the C++ {@code V*} / Rust {@code &mut V} write access.
     */
    public static final class ValueRef<V> {

        private final CoordSpace<V> space;
        private final Coord coord;

        private ValueRef(CoordSpace<V> space, Coord coord) {
            this.space = space;
            this.coord = coord;
        }

        /** The referenced coordinate. */
        public Coord coord() {
            return coord;
        }

        /** The current value; throws when the slot has been vacated. */
        public V get() {
            return space.at(coord).orElseThrow(
                    () -> new NoSuchElementException("CoordSpace slot vacant: " + coord));
        }

        /** Replaces the stored value; throws when the slot has been vacated. */
        public void set(V value) {
            Objects.requireNonNull(value, "null value");
            if (!space.occupied(coord)) {
                throw new NoSuchElementException("CoordSpace slot vacant: " + coord);
            }
            space.place(coord, value);
        }

        /** Applies {@code updater} to the stored value and writes it back. */
        public V update(UnaryOperator<V> updater) {
            V updated = Objects.requireNonNull(updater.apply(get()), "updater returned null");
            set(updated);
            return updated;
        }
    }

    /**
     * An entry handle for a {@link CoordSpace} slot, mirroring the C++ inner
     * {@code Entry} and the Rust {@code FlatEntry}.
     */
    public static final class Entry<V> {

        private final CoordSpace<V> space;
        private final Coord coord;

        private Entry(CoordSpace<V> space, Coord coord) {
            this.space = space;
            this.coord = coord;
        }

        /**
         * Returns a mutable handle to the value at this entry's coordinate,
         * inserting {@code value} when the slot is vacant. Never overwrites an
         * existing value.
         */
        public ValueRef<V> orInsert(V value) {
            Objects.requireNonNull(value, "null value");
            if (!space.occupied(coord)) {
                space.place(coord, value);
            }
            return new ValueRef<>(space, coord);
        }

        /** Like {@link #orInsert(Object)} but computes the value lazily. */
        public ValueRef<V> orInsertWith(Supplier<V> factory) {
            return orInsert(Objects.requireNonNull(factory.get(), "factory returned null"));
        }
    }
}
