package org.ssccs.syntagma.map;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.Objects;
import java.util.Optional;

import org.ssccs.syntagma.core.Coord;
import org.ssccs.syntagma.core.CoordCube;
import org.ssccs.syntagma.core.CoordPath;
import org.ssccs.syntagma.geo.BoundingBoxIter;
import org.ssccs.syntagma.geo.SpatialOps;

/**
 * Spatial queries over the coordinate maps: keys are interpreted as
 * multi-dimensional coordinates through {@link CoordCube}, the query region is
 * generated with {@code tagma-geo}, and matching entries are looked up through
 * {@link CoordPathLookup}.
 *
 * <p>This is the static facade that replaces the Rust {@code CoordCubeMap}
 * extension trait and the C++ free functions of the same names. Java has no
 * const generics, so the character count {@code N} and the cube interpretation
 * ({@code D}, {@code R}) are call-site arguments, and the store is passed as
 * {@link CoordPathLookup} so its size is known at runtime through the looked-up
 * path.
 *
 * <p>Queries operate in the byte-space domain of the store keys, one byte per
 * character: a center character or a range bound at or above
 * {@link CoordKey#BYTE_DOMAIN} is a caller error and is rejected before any
 * generation runs. A radius that crosses the domain edge clamps at the edge
 * instead of wrapping onto low byte values, because generation is bounded
 * through {@link SpatialOps#proximityBounded(CoordCube, int, int)}.
 *
 * <p>Port of the C++ free functions in
 * {@code sw/cpp/tagma_map/include/tagma_map/coord_cube_map.h}; the underlying
 * behavior mirrors the Rust {@code CoordCubeMap} trait in
 * {@code sw/rust/map/src/coord_cube_map.rs}.
 */
public final class CoordCubeMap {

    /**
     * The largest list capacity requested up front. Java list capacities are
     * {@code int}, and an allocation at the array-size limit throws
     * {@link OutOfMemoryError} before a single path is stored, so the hint is
     * clamped instead of being scaled with the query area. Beyond this ceiling
     * the amortised growth of the result list costs less than a speculative
     * allocation of the same size.
     */
    private static final int MAX_CAPACITY_HINT = 1 << 20;

    private CoordCubeMap() {
    }

    /**
     * One matched entry: the generated path and its stored value.
     *
     * <p>Replaces the C++ {@code std::pair<CoordPath, std::vector<uint8_t>>}
     * and the Rust {@code (CoordPath, Vec<u8>)} tuple. Equality is value-based,
     * as for those two pair types, because a Java record would otherwise
     * compare the value arrays by identity. The array is the store's defensive
     * copy; the caller owns it and may mutate it without affecting the store.
     */
    public record Hit(CoordPath path, byte[] value) {

        public Hit {
            Objects.requireNonNull(path, "path");
            Objects.requireNonNull(value, "value");
        }

        @Override
        public boolean equals(Object other) {
            if (this == other) {
                return true;
            }
            if (!(other instanceof Hit hit)) {
                return false;
            }
            return path.equals(hit.path) && Arrays.equals(value, hit.value);
        }

        @Override
        public int hashCode() {
            return 31 * path.hashCode() + Arrays.hashCode(value);
        }

        @Override
        public String toString() {
            return "Hit[" + path + ", " + value.length + " byte(s)]";
        }
    }

    /**
     * All entries within the L-infinity (Chebyshev) distance {@code radius} of
     * {@code center}, interpreted as a {@link CoordCube} of {@code dimensions}
     * by {@code resolution} characters.
     *
     * <p>Generation is bounded to {@link CoordKey#BYTE_DOMAIN}, so a radius
     * that crosses the domain edge clamps instead of wrapping. The result is
     * pre-sized with the exact number of paths the bounded region generates, so
     * a radius far beyond the byte domain cannot inflate the allocation.
     *
     * <p>A center that cannot address the store yields an empty result rather
     * than an exception: a path whose length differs from the path length of
     * the store is absent by definition, as
     * {@link CoordPathLookup#getByCoordPath(CoordPath)} documents. A center
     * whose length is not {@code dimensions * resolution} is still a caller
     * error and is rejected.
     *
     * @throws IllegalArgumentException when a center character index is at or
     *         above {@link CoordKey#BYTE_DOMAIN}, when {@code dimensions} or
     *         {@code resolution} is less than 1, when {@code dimensions *
     *         resolution} differs from the center length, or when
     *         {@code radius} is negative
     * @throws NullPointerException when {@code map} or {@code center} is null
     */
    public static List<Hit> proximity(CoordPathLookup map, CoordPath center, int radius,
            int dimensions, int resolution) {
        Objects.requireNonNull(map, "map");
        Objects.requireNonNull(center, "center");
        requireCenterInByteDomain(center);
        requirePositiveInterpretation(dimensions, resolution);
        CoordCube cube = CoordCube.fromPath(dimensions, resolution, center);
        BoundingBoxIter box = SpatialOps.proximityBounded(cube, radius, CoordKey.BYTE_DOMAIN);
        List<Hit> results = new ArrayList<>(capacityHint(box.countPaths()));
        for (CoordPath path : box) {
            Optional<byte[]> value = map.getByCoordPath(path);
            if (value.isPresent()) {
                results.add(new Hit(path, value.orElseThrow()));
            }
        }
        return results;
    }

    /**
     * All entries within a bounding box defined by per-character
     * {@code [min, max]} ranges. The range count fixes the path length of the
     * query, the Java counterpart of the deduced array size of the C++ and
     * Rust signatures.
     *
     * <p>The result is pre-sized with the exact path count of
     * {@link BoundingBoxIter#countPaths()}, which saturates on overflow, as in
     * the C++ port, and is then clamped to
     * {@link #MAX_CAPACITY_HINT}.
     *
     * <p>A range set whose length differs from the path length of the store
     * yields an empty result rather than an exception: the generated paths are
     * absent from a store that cannot hold them, as
     * {@link CoordPathLookup#getByCoordPath(CoordPath)} documents.
     *
     * @throws IllegalArgumentException when a range bound is outside
     *         {@code [0, CoordKey.BYTE_DOMAIN)}, when a range is not a
     *         {@code [min, max]} pair, or when a range is invalid
     * @throws NullPointerException when {@code map} or {@code ranges} is null
     */
    public static List<Hit> boundingBoxRange(CoordPathLookup map, int[][] ranges) {
        Objects.requireNonNull(map, "map");
        requireRangesInByteDomain(ranges);
        BoundingBoxIter box = new BoundingBoxIter(ranges);
        List<Hit> results = new ArrayList<>(capacityHint(box.countPaths()));
        for (CoordPath path : box) {
            Optional<byte[]> value = map.getByCoordPath(path);
            if (value.isPresent()) {
                results.add(new Hit(path, value.orElseThrow()));
            }
        }
        return results;
    }

    /**
     * Rejects a dimension count that is not positive. The references carry both
     * counts as unsigned template parameters, so a non-positive count cannot be
     * expressed there; Java would otherwise accept a negative pair whose
     * product happens to match the center length, and would accept a degenerate
     * zero-sized query that the caller cannot have meant.
     */
    private static void requirePositiveInterpretation(int dimensions, int resolution) {
        if (dimensions < 1 || resolution < 1) {
            throw new IllegalArgumentException(
                    "CoordCubeMap::proximity: dimensions " + dimensions + " and resolution "
                            + resolution + " must both be at least 1");
        }
    }

    /**
     * Rejects a center character index at or above the byte-space domain.
     * Mirrors the always-on panic of the Rust port and the
     * {@code std::invalid_argument} of the C++ port.
     */
    private static void requireCenterInByteDomain(CoordPath center) {
        Coord[] coords = center.coords();
        for (int i = 0; i < coords.length; i++) {
            int index = coords[i].index();
            if (index >= CoordKey.BYTE_DOMAIN) {
                throw new IllegalArgumentException(
                        "CoordCubeMap: center character " + i + " index " + index
                                + " is outside the map byte-space domain [0, "
                                + CoordKey.BYTE_DOMAIN + ")");
            }
        }
    }

    /**
     * Rejects a range bound outside the byte-space domain, and a range that is
     * not a {@code [min, max]} pair, before any generation runs.
     */
    private static void requireRangesInByteDomain(int[][] ranges) {
        Objects.requireNonNull(ranges, "ranges");
        for (int i = 0; i < ranges.length; i++) {
            int[] range = ranges[i];
            if (range == null || range.length != 2) {
                throw new IllegalArgumentException(
                        "CoordCubeMap: range " + i + " must be a [min, max] pair");
            }
            int min = range[0];
            int max = range[1];
            if (min < 0 || min >= CoordKey.BYTE_DOMAIN
                    || max < 0 || max >= CoordKey.BYTE_DOMAIN) {
                throw new IllegalArgumentException(
                        "CoordCubeMap: range " + i + " (" + min + ", " + max
                                + ") is outside the map byte-space domain [0, "
                                + CoordKey.BYTE_DOMAIN + ")");
            }
        }
    }

    /**
     * The capacity requested up front for a result list, from the exact number
     * of paths the query will generate.
     *
     * <p>The Rust port pre-sizes proximity with the unclamped upper bound
     * {@code (2 * radius + 1) ^ N}; the bounded region of a store query is a
     * subset of that bound, and a radius far beyond the byte domain makes the
     * two differ by many orders of magnitude, so Java uses the count the
     * iterator actually yields and clamps it to {@link #MAX_CAPACITY_HINT} for
     * the {@code int} capacity of a Java list.
     */
    private static int capacityHint(long paths) {
        return (int) Math.min(paths, (long) MAX_CAPACITY_HINT);
    }
}
