package org.ssccs.syntagma.core;

import java.util.Arrays;
import java.util.Objects;

/**
 * An interpretation layer over a {@link CoordPath} that views {@code N}
 * characters as a {@code D}-dimensional grid with {@code R} characters of
 * resolution per dimension (11,172^R addressable values per dimension).
 *
 * <p>{@code N} must equal {@code D * R}; the Rust reference enforces this at
 * runtime in {@code from_path} and the C++ reference at compile time with
 * {@code static_assert}. Java carries the three dimensions as immutable
 * instance state and enforces the product constraint at construction.
 *
 * <p>{@code CoordCube} never modifies or replaces the underlying
 * {@code CoordPath}; storage always uses the path. It is an optional
 * interpretation layer over the same coordinates.
 *
 * <p>Port of the C++ {@code tagma::CoordCube<N, D, R>} in
 * {@code sw/cpp/tagma_core}; the underlying behavior mirrors the Rust
 * {@code CoordCube<N, D, R>} in {@code sw/rust/core}.
 */
public final class CoordCube {

    private final int dimensions;
    private final int resolution;
    private final CoordPath path;

    private CoordCube(int dimensions, int resolution, CoordPath path) {
        this.dimensions = dimensions;
        this.resolution = resolution;
        this.path = path;
    }

    /**
     * Creates a cube interpreting {@code path} as a {@code dimensions}-by-
     * {@code resolution} grid.
     *
     * @throws IllegalArgumentException when {@code path.length() != dimensions * resolution}
     */
    public static CoordCube fromPath(int dimensions, int resolution, CoordPath path) {
        Objects.requireNonNull(path, "path");
        int product = dimensions * resolution;
        if (path.length() != product) {
            throw new IllegalArgumentException(
                    "CoordCube: N=" + path.length() + " must equal D*R = " + dimensions
                            + "*" + resolution + " = " + product);
        }
        return new CoordCube(dimensions, resolution, path);
    }

    /** The number of spatial dimensions. */
    public int ndim() {
        return dimensions;
    }

    /** The number of characters per dimension. */
    public int resolution() {
        return resolution;
    }

    /** The total number of characters ({@code N == D * R}). */
    public int totalCharacters() {
        return path.length();
    }

    /**
     * The {@code R}-character path for dimension {@code dim}.
     *
     * @throws IllegalArgumentException when {@code dim} is outside [0, D)
     */
    public CoordPath axis(int dim) {
        requireDim(dim);
        Coord[] segment = new Coord[resolution];
        for (int i = 0; i < resolution; i++) {
            segment[i] = coordAt(dim, i);
        }
        return CoordPath.fromArrayUnchecked(segment);
    }

    /**
     * The {@code Coord} at {@code character} within dimension {@code dim}.
     *
     * @throws IllegalArgumentException when {@code dim} or {@code character} is out of range
     */
    public Coord coordAt(int dim, int character) {
        requireDim(dim);
        if (character < 0 || character >= resolution) {
            throw new IllegalArgumentException(
                    "CoordCube::coord_at: character " + character + " out of range [0, " + resolution + ")");
        }
        return path.get(dim * resolution + character).orElseThrow();
    }

    /** The full coordinate array as a defensive copy. */
    public Coord[] coords() {
        return path.coords();
    }

    /** The underlying path (immutable, safe to share). */
    public CoordPath asPath() {
        return path;
    }

    /** The underlying path. */
    public CoordPath intoPath() {
        return path;
    }

    private void requireDim(int dim) {
        if (dim < 0 || dim >= dimensions) {
            throw new IllegalArgumentException(
                    "CoordCube::axis: dim " + dim + " out of range [0, " + dimensions + ")");
        }
    }

    @Override
    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        if (!(other instanceof CoordCube cube)) {
            return false;
        }
        return dimensions == cube.dimensions
                && resolution == cube.resolution
                && path.equals(cube.path);
    }

    @Override
    public int hashCode() {
        return 31 * (31 * Integer.hashCode(dimensions) + Integer.hashCode(resolution)) + path.hashCode();
    }

    /** Displays the cube, mirroring the C++/Rust {@code Display} impl. */
    @Override
    public String toString() {
        StringBuilder out = new StringBuilder("CoordCube<")
                .append(totalCharacters()).append(", ").append(dimensions).append(", ")
                .append(resolution).append(">[");
        Coord[] all = coords();
        for (int dim = 0; dim < dimensions; dim++) {
            if (dim > 0) {
                out.append(" | ");
            }
            out.append('(');
            for (int i = 0; i < resolution; i++) {
                if (i > 0) {
                    out.append(", ");
                }
                out.append(all[dim * resolution + i]);
            }
            out.append(')');
        }
        return out.append(']').toString();
    }
}
