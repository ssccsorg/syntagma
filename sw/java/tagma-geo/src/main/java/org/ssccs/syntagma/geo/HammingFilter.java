package org.ssccs.syntagma.geo;

import java.util.Iterator;
import java.util.NoSuchElementException;
import java.util.Objects;

import org.ssccs.syntagma.core.Coord;
import org.ssccs.syntagma.core.CoordPath;

/**
 * A view over a {@link BoundingBoxIter} that yields only the paths within a
 * Hamming radius of a center path, where the distance is the number of
 * character positions that differ.
 *
 * <p>Java has no const generics, so the shared length tag of the C++/Rust
 * {@code HammingFilter<N>} becomes a runtime check: the center length must
 * equal the underlying iterator length. As with {@link BoundingBoxIter}, an
 * instance acts as an {@link Iterator} and, through {@link #iterator()}, as an
 * {@link Iterable} that hands out independent position copies.
 *
 * <p>Port of the C++ {@code tagma_geo::HammingFilter<N>} in
 * {@code sw/cpp/tagma_geo/include/tagma_geo/spatial.h}; the underlying
 * behavior mirrors the Rust {@code HammingFilter<N>} in
 * {@code sw/rust/geo/src/spatial.rs}.
 */
public final class HammingFilter implements Iterable<CoordPath>, Iterator<CoordPath> {

    private final BoundingBoxIter inner;

    /**
     * Snapshot of the center coordinates. {@code CoordPath.coords()} returns a
     * defensive copy, so taking it once here keeps a clone per candidate out
     * of the filter loop.
     */
    private final Coord[] centerCoords;

    private final int maxDistance;

    /** The next matching path, when a previous look-ahead already found it. */
    private CoordPath pending;

    /**
     * Creates a filter over {@code inner} around {@code center}.
     *
     * @param inner the underlying bounding box iterator
     * @param center the center path; zero distance means "differs in no
     *        character position"
     * @param maxDistance the largest Hamming distance to yield
     * @throws IllegalArgumentException when the center length differs from the
     *         iterator length or {@code maxDistance} is negative
     */
    public HammingFilter(BoundingBoxIter inner, CoordPath center, int maxDistance) {
        this(Objects.requireNonNull(inner, "inner"),
                Objects.requireNonNull(center, "center").coords(), maxDistance, null);
    }

    /**
     * Internal constructor. Both arguments are already non-null when this
     * runs: the public constructor checks them in parameter order and
     * {@link #iterator()} passes validated state. The fields are assigned
     * after every check, so a rejected construction touches no state.
     */
    private HammingFilter(BoundingBoxIter inner, Coord[] centerCoords, int maxDistance, CoordPath pending) {
        if (centerCoords.length != inner.ndim()) {
            throw new IllegalArgumentException(
                    "HammingFilter: center length " + centerCoords.length
                            + " must equal the iterator length " + inner.ndim());
        }
        if (maxDistance < 0) {
            throw new IllegalArgumentException(
                    "HammingFilter: max distance " + maxDistance + " must not be negative");
        }
        this.inner = inner;
        this.centerCoords = centerCoords;
        this.maxDistance = maxDistance;
        this.pending = pending;
    }

    /** Whether another path within the Hamming radius remains to be yielded. */
    @Override
    public boolean hasNext() {
        if (pending != null) {
            return true;
        }
        while (inner.hasNext()) {
            CoordPath candidate = inner.next();
            if (hammingToCenter(candidate) <= maxDistance) {
                pending = candidate;
                return true;
            }
        }
        return false;
    }

    /**
     * The next path within the Hamming radius.
     *
     * @throws NoSuchElementException when the filter is exhausted
     */
    @Override
    public CoordPath next() {
        if (!hasNext()) {
            throw new NoSuchElementException("HammingFilter exhausted");
        }
        CoordPath result = pending;
        pending = null;
        return result;
    }

    /**
     * An independent copy positioned where this instance currently stands,
     * including any path already found by a look-ahead.
     */
    @Override
    public Iterator<CoordPath> iterator() {
        return new HammingFilter(inner.copy(), centerCoords, maxDistance, pending);
    }

    private int hammingToCenter(CoordPath candidate) {
        Coord[] candidateCoords = candidate.coords();
        int distance = 0;
        for (int i = 0; i < candidateCoords.length; i++) {
            if (!candidateCoords[i].equals(centerCoords[i])) {
                distance++;
            }
        }
        return distance;
    }
}
