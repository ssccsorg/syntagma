package org.ssccs.syntagma.geo;

import java.util.Iterator;
import java.util.NoSuchElementException;
import java.util.Objects;

import org.ssccs.syntagma.core.Coord;
import org.ssccs.syntagma.core.CoordPath;

/**
 * An iterator over every {@link CoordPath} whose character {@code i} lies in
 * the range {@code ranges[i] = [min, max]}, yielded in mixed-radix order.
 *
 * <p>Java has no const generics, so the compile-time length tag of the
 * C++/Rust {@code BoundingBoxIter<N>} becomes immutable instance state: the
 * range array length fixes {@code N} at construction. The unsigned bound type
 * of the references ({@code uint16_t} / {@code u16}) becomes {@code int} with
 * an explicit non-negativity check, because a Java {@code int} can carry
 * values those types cannot represent.
 *
 * <p>An instance serves both roles of the reference iteration protocol: as an
 * {@link Iterator} it drives the mixed-radix counter in place, and as an
 * {@link Iterable} it hands out independent copies positioned where this
 * instance currently stands, which mirrors the value semantics of the C++
 * {@code begin()} and {@code end()} pair.
 *
 * <p>Port of the C++ {@code tagma_geo::BoundingBoxIter<N>} in
 * {@code sw/cpp/tagma_geo/include/tagma_geo/spatial.h}; the underlying
 * behavior mirrors the Rust {@code BoundingBoxIter<N>} in
 * {@code sw/rust/geo/src/spatial.rs}.
 */
public final class BoundingBoxIter implements Iterable<CoordPath>, Iterator<CoordPath> {

    private final int ndim;
    private final int[][] ranges;
    private final int[] current;
    private boolean finished;

    /**
     * Creates an iterator over the given per-character ranges. Ranges are
     * copied, so later changes to {@code ranges} do not affect the iterator.
     *
     * @param ranges per-character {@code [min, max]} pairs; the array length
     *        is {@code N}
     * @throws IllegalArgumentException when a range is not a two-element pair,
     *         when {@code min} is negative, when {@code min > max}, or when
     *         {@code max} is at or above {@link Coord#N_VALID}
     */
    public BoundingBoxIter(int[][] ranges) {
        Objects.requireNonNull(ranges, "ranges");
        this.ndim = ranges.length;
        this.ranges = new int[ndim][2];
        for (int i = 0; i < ndim; i++) {
            int[] range = ranges[i];
            if (range == null || range.length != 2) {
                throw new IllegalArgumentException(
                        "BoundingBoxIter: range " + i + " must be a [min, max] pair");
            }
            int min = range[0];
            int max = range[1];
            if (min < 0) {
                throw new IllegalArgumentException(
                        "BoundingBoxIter: range " + i + " has negative min " + min);
            }
            if (min > max) {
                throw new IllegalArgumentException(
                        "BoundingBoxIter: range " + i + " has min " + min + " > max " + max);
            }
            if (max >= Coord.N_VALID) {
                throw new IllegalArgumentException(
                        "BoundingBoxIter: range " + i + " has max " + max + " >= " + Coord.N_VALID);
            }
            this.ranges[i][0] = min;
            this.ranges[i][1] = max;
        }
        this.current = new int[ndim];
        for (int i = 0; i < ndim; i++) {
            current[i] = this.ranges[i][0];
        }
        this.finished = ndim == 0;
    }

    /**
     * Internal constructor for {@link #iterator()}. The validated range array
     * is shared: it is never mutated after construction, only {@code current}
     * and {@code finished} advance.
     */
    private BoundingBoxIter(int[][] sharedRanges, int[] position, boolean exhausted) {
        this.ndim = sharedRanges.length;
        this.ranges = sharedRanges;
        this.current = position;
        this.finished = exhausted;
    }

    /** The number of characters per path ({@code N}). */
    public int ndim() {
        return ndim;
    }

    /** Whether no path remains to be yielded. */
    public boolean isEmpty() {
        return finished;
    }

    /**
     * The total number of paths, the product of the range widths, saturating
     * at {@code Long.MAX_VALUE} on overflow, mirroring the saturating
     * arithmetic of the C++ and Rust references. For {@code N == 0} the count
     * is 0 rather than the vacuous product 1, as in the C++ reference.
     *
     * <p>Java has no unsigned 64-bit integer, so the saturation point is
     * {@code Long.MAX_VALUE} where {@code std::size_t} and {@code usize}
     * saturate at their own maximum.
     */
    public long countPaths() {
        if (ndim == 0) {
            return 0L;
        }
        long total = 1L;
        for (int i = 0; i < ndim; i++) {
            long width = (long) ranges[i][1] - ranges[i][0] + 1L;
            total = saturatingMul(total, width);
        }
        return total;
    }

    /** Whether a path remains to be yielded. */
    @Override
    public boolean hasNext() {
        return !finished;
    }

    /**
     * The next path in mixed-radix order.
     *
     * @throws NoSuchElementException when the iterator is exhausted
     */
    @Override
    public CoordPath next() {
        if (finished) {
            throw new NoSuchElementException("BoundingBoxIter exhausted");
        }
        Coord[] coords = new Coord[ndim];
        for (int i = 0; i < ndim; i++) {
            coords[i] = Coord.fromIndex(current[i]).orElseThrow();
        }
        CoordPath path = CoordPath.fromArray(coords);

        int position = ndim;
        while (position > 0) {
            position--;
            if (current[position] < ranges[position][1]) {
                current[position]++;
                for (int reset = position + 1; reset < ndim; reset++) {
                    current[reset] = ranges[reset][0];
                }
                return path;
            }
            current[position] = ranges[position][0];
        }
        finished = true;
        return path;
    }

    /**
     * An independent copy positioned where this instance currently stands, so
     * that each call yields the remaining paths of this iterator without
     * disturbing it.
     */
    @Override
    public Iterator<CoordPath> iterator() {
        return copy();
    }

    /**
     * Package-private position copy, the value-semantics hand-off used by
     * {@link HammingFilter#iterator()}.
     */
    BoundingBoxIter copy() {
        return new BoundingBoxIter(ranges, current.clone(), finished);
    }

    private static long saturatingMul(long a, long b) {
        return b != 0 && a > Long.MAX_VALUE / b ? Long.MAX_VALUE : a * b;
    }
}
