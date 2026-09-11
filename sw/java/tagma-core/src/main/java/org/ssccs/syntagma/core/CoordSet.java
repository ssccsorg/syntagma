package org.ssccs.syntagma.core;

import java.util.BitSet;
import java.util.Iterator;
import java.util.Optional;
import java.util.function.Predicate;

/**
 * A fixed-size, collision-free bit array for presence checking of
 * {@link Coord} values.
 *
 * <p>Backed by a bit set over the 11,172 valid slots (about 1.4 KB). No
 * hashing, no collisions. Insert, remove, and membership are single-bit
 * operations; the set operations are bitwise over the whole space.
 *
 * <p>Port of the C++ {@code tagma::CoordSet} in {@code sw/cpp/tagma_core};
 * the underlying behavior mirrors the Rust {@code CoordSet} in
 * {@code sw/rust/core}. The C++ {@code std::bitset} backing is implemented
 * with {@link BitSet}, which exposes the same observable operations.
 */
public final class CoordSet implements Iterable<Coord> {

    /** The maximum number of elements the set can hold, always 11,172. */
    public static final int CAPACITY = Coord.N_VALID;

    private final BitSet bits;

    /** Creates an empty {@code CoordSet}. */
    public CoordSet() {
        bits = new BitSet(CAPACITY);
    }

    private CoordSet(BitSet bits) {
        this.bits = bits;
    }

    /** Returns an independent copy of this set. */
    public CoordSet copy() {
        return new CoordSet((BitSet) bits.clone());
    }

    /**
     * Inserts {@code coord} into the set.
     *
     * @return {@code true} if {@code coord} was not already present
     */
    public boolean insert(Coord coord) {
        int index = coord.index();
        if (bits.get(index)) {
            return false;
        }
        bits.set(index);
        return true;
    }

    /**
     * Removes {@code coord} from the set.
     *
     * @return {@code true} if {@code coord} was present
     */
    public boolean remove(Coord coord) {
        int index = coord.index();
        boolean present = bits.get(index);
        bits.clear(index);
        return present;
    }

    /** Returns {@code true} if {@code coord} is in the set. */
    public boolean contains(Coord coord) {
        return bits.get(coord.index());
    }

    /** Clears all elements from the set. */
    public void clear() {
        bits.clear();
    }

    /** Returns the number of elements in the set (popcount). */
    public int size() {
        return bits.cardinality();
    }

    /** Returns {@code true} if the set contains no elements. */
    public boolean isEmpty() {
        return bits.isEmpty();
    }

    /** The maximum number of elements, always 11,172. */
    public static int capacity() {
        return CAPACITY;
    }

    /** The union of this set and {@code other} (elements in either). */
    public CoordSet union(CoordSet other) {
        BitSet result = (BitSet) bits.clone();
        result.or(other.bits);
        return new CoordSet(result);
    }

    /** The intersection of this set and {@code other} (elements in both). */
    public CoordSet intersection(CoordSet other) {
        BitSet result = (BitSet) bits.clone();
        result.and(other.bits);
        return new CoordSet(result);
    }

    /** The difference {@code this \ other} (elements in this but not other). */
    public CoordSet difference(CoordSet other) {
        BitSet result = (BitSet) bits.clone();
        result.andNot(other.bits);
        return new CoordSet(result);
    }

    /** The symmetric difference (elements in exactly one of the two sets). */
    public CoordSet symmetricDifference(CoordSet other) {
        BitSet result = (BitSet) bits.clone();
        result.xor(other.bits);
        return new CoordSet(result);
    }

    /** Returns {@code true} if every element of this set is in {@code other}. */
    public boolean isSubset(CoordSet other) {
        BitSet outside = (BitSet) bits.clone();
        outside.andNot(other.bits);
        return outside.isEmpty();
    }

    /** Returns {@code true} if this set contains every element of {@code other}. */
    public boolean isSuperset(CoordSet other) {
        return other.isSubset(this);
    }

    /** Returns {@code true} if the sets share no element. */
    public boolean isDisjoint(CoordSet other) {
        BitSet common = (BitSet) bits.clone();
        common.and(other.bits);
        return common.isEmpty();
    }

    /**
     * Returns the coordinate when present, mirroring a set {@code get}.
     */
    public Optional<Coord> get(Coord coord) {
        return contains(coord) ? Optional.of(coord) : Optional.empty();
    }

    /**
     * Removes and returns the coordinate when present, mirroring a set
     * {@code take}.
     */
    public Optional<Coord> take(Coord coord) {
        return remove(coord) ? Optional.of(coord) : Optional.empty();
    }

    /** Retains only the coordinates satisfying {@code predicate}. */
    public void retain(Predicate<Coord> predicate) {
        for (int index = bits.nextSetBit(0); index >= 0; index = bits.nextSetBit(index + 1)) {
            if (!predicate.test(Coord.raw(index))) {
                bits.clear(index);
            }
        }
    }

    /** Iterates over the present coordinates in index order. */
    @Override
    public Iterator<Coord> iterator() {
        return new Iterator<>() {
            private int current = bits.nextSetBit(0);

            @Override
            public boolean hasNext() {
                return current >= 0;
            }

            @Override
            public Coord next() {
                if (current < 0) {
                    throw new java.util.NoSuchElementException("CoordSet iterator exhausted");
                }
                Coord coord = Coord.raw(current);
                current = bits.nextSetBit(current + 1);
                return coord;
            }
        };
    }

    @Override
    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        if (!(other instanceof CoordSet set)) {
            return false;
        }
        return bits.equals(set.bits);
    }

    @Override
    public int hashCode() {
        return bits.hashCode();
    }

    /** Displays the set, mirroring the Rust {@code Display} impl. */
    @Override
    public String toString() {
        StringBuilder out = new StringBuilder("{");
        boolean first = true;
        for (Coord coord : this) {
            if (!first) {
                out.append(", ");
            }
            out.append(coord);
            first = false;
        }
        return out.append('}').toString();
    }
}
