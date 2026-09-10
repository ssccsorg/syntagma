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
     * pre-sized with the saturating capacity computation of the Rust port,
     * {@code (2 * radius + 1) ^ N}.
     *
     * @throws IllegalArgumentException when a center character index is at or
     *         above {@link CoordKey#BYTE_DOMAIN}, when {@code dimensions *
     *         resolution} differs from the center length, when
     *         {@code radius} is negative, or when a dimension count is not
     *         positive
     * @throws NullPointerException when {@code map} or {@code center} is null
     */
    public static List<Hit> proximity(CoordPathLookup map, CoordPath center, int radius,
            int dimensions, int resolution) {
        Objects.requireNonNull(map, "map");
        Objects.requireNonNull(center, "center");
        requireCenterInByteDomain(center);
        CoordCube cube = CoordCube.fromPath(dimensions, resolution, center);
        BoundingBoxIter box = SpatialOps.proximityBounded(cube, radius, CoordKey.BYTE_DOMAIN);
        List<Hit> results = new ArrayList<>(capacityHint(radius, center.length()));
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
     * the C++ port.
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
     * The saturating capacity computation of the Rust port,
     * {@code (2 * radius + 1) ^ characters}, clamped to the {@code int}
     * capacity of a Java list. A saturating value above
     * {@link Integer#MAX_VALUE} would exhaust memory during generation long
     * before the exact capacity mattered.
     */
    private static int capacityHint(int radius, int characters) {
        long base = saturatingAdd(saturatingMul(2L, radius), 1L);
        return capacityHint(saturatingPow(base, characters));
    }

    private static int capacityHint(long paths) {
        return (int) Math.min(paths, Integer.MAX_VALUE);
    }

    private static long saturatingMul(long a, long b) {
        if (a == 0 || b == 0) {
            return 0L;
        }
        return a > Long.MAX_VALUE / b ? Long.MAX_VALUE : a * b;
    }

    private static long saturatingAdd(long a, long b) {
        return a > Long.MAX_VALUE - b ? Long.MAX_VALUE : a + b;
    }

    private static long saturatingPow(long base, int exponent) {
        long result = 1L;
        for (int i = 0; i < exponent; i++) {
            result = saturatingMul(result, base);
        }
        return result;
    }
}
