package org.ssccs.syntagma.core;

import java.util.Arrays;
import java.util.Collections;
import java.util.Iterator;
import java.util.List;
import java.util.Objects;
import java.util.Optional;

/**
 * An index path through a coordinate space, not a key.
 *
 * <p>Each element selects one of 11,172 slots at the corresponding tree depth.
 * A {@code CoordPath} is not a hash map key; each coordinate is used directly
 * as an array index at one tree level, so no hashing and no equality
 * comparison is involved.
 *
 * <p>Java has no const generics, so the C++/Rust compile-time length tag
 * ({@code CoordPath<N>}) becomes immutable instance state with a runtime
 * check. {@link #coords()} returns a defensive copy to preserve value
 * semantics.
 *
 * <p>Port of the C++ {@code tagma::CoordPath<N>} in
 * {@code sw/cpp/tagma_core}; the underlying behavior mirrors the Rust
 * {@code CoordPath<N>} in {@code sw/rust/core}.
 */
public final class CoordPath implements Iterable<Coord> {

    private final Coord[] coords;

    private CoordPath(Coord[] coords) {
        this.coords = coords;
    }

    /**
     * Creates a path from an array of coordinates. The returned path holds a
     * defensive copy.
     *
     * @throws NullPointerException when the array or any element is null
     */
    public static CoordPath fromArray(Coord... coords) {
        Objects.requireNonNull(coords, "coords array");
        for (Coord coord : coords) {
            Objects.requireNonNull(coord, "null coordinate in path");
        }
        return new CoordPath(coords.clone());
    }

    /**
     * Creates a path from an array of coordinates, without copying. Caller
     * must not mutate {@code coords} afterwards.
     */
    static CoordPath fromArrayUnchecked(Coord[] coords) {
        return new CoordPath(coords);
    }

    /** The number of coordinates in the path. */
    public int length() {
        return coords.length;
    }

    /** Whether the path holds no coordinates ({@code length() == 0}). */
    public boolean isEmpty() {
        return coords.length == 0;
    }

    /**
     * The internal coordinate array as a defensive copy.
     */
    public Coord[] coords() {
        return coords.clone();
    }

    /**
     * The coordinate at {@code index}.
     *
     * @return empty when {@code index} is out of range
     */
    public Optional<Coord> get(int index) {
        return index >= 0 && index < coords.length
                ? Optional.of(coords[index])
                : Optional.empty();
    }

    /** Iterates over the coordinates in path order. */
    @Override
    public Iterator<Coord> iterator() {
        List<Coord> view = Collections.unmodifiableList(Arrays.asList(coords));
        return view.iterator();
    }

    @Override
    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        if (!(other instanceof CoordPath path)) {
            return false;
        }
        return Arrays.equals(coords, path.coords);
    }

    @Override
    public int hashCode() {
        return Arrays.hashCode(coords);
    }

    /** Displays the path, mirroring the C++ stream output. */
    @Override
    public String toString() {
        StringBuilder out = new StringBuilder("CoordPath<").append(coords.length).append(">(");
        for (int i = 0; i < coords.length; i++) {
            if (i > 0) {
                out.append(", ");
            }
            out.append(coords[i]);
        }
        return out.append(')').toString();
    }
}
