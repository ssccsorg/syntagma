package org.ssccs.syntagma.geo;

import org.ssccs.syntagma.core.Coord;
import org.ssccs.syntagma.core.CoordCube;

/**
 * Spatial region generation and distance measurement over {@link CoordCube}.
 *
 * <p>This is the static facade that replaces the Rust {@code SpatialOps} and
 * {@code DistanceMetrics} traits and the C++ free functions of the same names.
 * Region generation is bounded by a per-character domain: the full-domain
 * {@link #proximity(CoordCube, int)} delegates to the domain-bounded
 * {@link #proximityBounded(CoordCube, int, int)}, and storage layers whose keys
 * occupy a narrower domain, such as a byte-space store over {@code [0, 256)},
 * must use the bounded entry point so generated regions stay inside the store
 * domain.
 *
 * <p>Dimension values are little-endian base-11172 integers. For a resolution
 * of 5 or more the 64-bit accumulation wraps, which matches the documented
 * reference limitation; the wrapped bit pattern is read with unsigned
 * semantics wherever it is widened to {@code double}.
 *
 * <p>Port of the C++ free functions in
 * {@code sw/cpp/tagma_geo/include/tagma_geo/spatial.h}; the underlying
 * behavior mirrors the Rust {@code SpatialOps} and {@code DistanceMetrics}
 * traits in {@code sw/rust/geo/src/spatial.rs}.
 */
public final class SpatialOps {

    private SpatialOps() {
    }

    /**
     * All paths within the given per-character {@code [min, max]} ranges.
     *
     * <p>The reference takes the ranges as a fixed-length array of length
     * {@code N}, the cube character count, so a mismatch is impossible to
     * express there. Java carries the length at runtime and rejects a
     * mismatch here.
     *
     * @throws IllegalArgumentException when the range count differs from the
     *         cube character count, or when a range is invalid
     */
    public static BoundingBoxIter boundingBox(CoordCube cube, int[][] ranges) {
        int characters = cube.totalCharacters();
        if (ranges == null || ranges.length != characters) {
            throw new IllegalArgumentException(
                    "SpatialOps::boundingBox: range count " + (ranges == null ? 0 : ranges.length)
                            + " must equal the cube character count " + characters);
        }
        return new BoundingBoxIter(ranges);
    }

    /**
     * All paths within an L-infinity (Chebyshev) proximity radius of the cube
     * center, clamped to the per-character domain {@code [0, domain)}.
     *
     * <p>Each character window is
     * {@code [max(0, index - radius), min(domain - 1, index + radius)]}, so a
     * radius that crosses a domain edge clamps at the edge and never wraps
     * onto low index values.
     *
     * <p>The references take an unsigned radius and domain, so negative values
     * are not expressible there; Java rejects them explicitly.
     *
     * @throws IllegalArgumentException when {@code radius} is negative, when
     *         {@code domain} is outside {@code (0, Coord.N_VALID]}, or when a
     *         center character index is at or above {@code domain}
     */
    public static BoundingBoxIter proximityBounded(CoordCube cube, int radius, int domain) {
        if (radius < 0) {
            throw new IllegalArgumentException(
                    "SpatialOps::proximityBounded: radius " + radius + " must not be negative");
        }
        if (domain <= 0 || domain > Coord.N_VALID) {
            throw new IllegalArgumentException(
                    "SpatialOps::proximityBounded: domain " + domain
                            + " must be in (0, " + Coord.N_VALID + "]");
        }
        int characters = cube.totalCharacters();
        int[][] ranges = new int[characters][2];
        Coord[] coords = cube.coords();
        for (int i = 0; i < characters; i++) {
            int index = coords[i].index();
            if (index >= domain) {
                throw new IllegalArgumentException(
                        "SpatialOps::proximityBounded: center character " + i + " index " + index
                                + " is at or above domain " + domain);
            }
            long lower = Math.max(0L, (long) index - radius);
            long upper = Math.min((long) domain - 1L, (long) index + radius);
            ranges[i][0] = (int) lower;
            ranges[i][1] = (int) upper;
        }
        return new BoundingBoxIter(ranges);
    }

    /**
     * All paths within an L-infinity (Chebyshev) proximity radius of the cube
     * center, clamped to the full {@link Coord} index domain
     * {@code [0, Coord.N_VALID)}.
     */
    public static BoundingBoxIter proximity(CoordCube cube, int radius) {
        return proximityBounded(cube, radius, Coord.N_VALID);
    }

    /**
     * All paths within a Hamming distance {@code radius} of the cube center.
     *
     * <p>The underlying box uses a radius of at least 1 so that a zero radius
     * still covers the center character positions.
     *
     * @throws IllegalArgumentException when {@code radius} is negative
     */
    public static HammingFilter proximityHamming(CoordCube cube, int radius) {
        if (radius < 0) {
            throw new IllegalArgumentException(
                    "SpatialOps::proximityHamming: radius " + radius + " must not be negative");
        }
        BoundingBoxIter box = proximity(cube, Math.max(radius, 1));
        return new HammingFilter(box, cube.asPath(), radius);
    }

    /**
     * The {@code resolution} characters of dimension {@code dim} as a
     * little-endian base-11172 integer.
     *
     * <p>The value wraps modulo 2^64 for a resolution of 5 or more, matching
     * the documented reference limitation. In Java the wrap is the natural
     * two's-complement overflow of {@code long}; the bit pattern is the same
     * as the 64-bit unsigned reference value.
     *
     * @throws IllegalArgumentException when {@code dim} is outside
     *         {@code [0, cube.ndim())}
     */
    public static long dimensionValue(CoordCube cube, int dim) {
        if (dim < 0 || dim >= cube.ndim()) {
            throw new IllegalArgumentException(
                    "SpatialOps::dimensionValue: dim " + dim + " out of range [0, " + cube.ndim() + ")");
        }
        Coord[] coords = cube.coords();
        int resolution = cube.resolution();
        int start = dim * resolution;
        long value = 0L;
        long multiplier = 1L;
        for (int i = 0; i < resolution; i++) {
            value += (long) coords[start + i].index() * multiplier;
            multiplier *= Coord.N_VALID;
        }
        return value;
    }

    /**
     * The maximum value of a single dimension, {@code 11172^resolution - 1}.
     * For a resolution of 5 or more the accumulation wraps modulo 2^64 exactly
     * as the reference code does. The C++ and Rust doc comments claim a result
     * of zero for that range, which their code does not produce: 11172^5 - 1
     * reduced modulo 2^64 is 8021531685948761087, and that wrapped value is
     * what the test suite pins.
     *
     * @throws IllegalArgumentException when {@code resolution} is negative
     */
    public static long dimensionMaxValue(int resolution) {
        if (resolution < 0) {
            throw new IllegalArgumentException(
                    "SpatialOps::dimensionMaxValue: resolution " + resolution + " must not be negative");
        }
        long maximum = 0L;
        long multiplier = 1L;
        long maxIndex = Coord.N_VALID - 1L;
        for (int i = 0; i < resolution; i++) {
            maximum += maxIndex * multiplier;
            multiplier *= Coord.N_VALID;
        }
        return maximum;
    }

    /**
     * The Hamming distance between two cubes: the number of character
     * positions that differ.
     *
     * <p>The references require both cubes to share the const generic
     * parameters {@code N}, {@code D} and {@code R}, which Java carries as
     * runtime state and checks here.
     *
     * @throws IllegalArgumentException when the cubes do not share
     *         {@code N}, {@code D} and {@code R}
     */
    public static int hammingDistance(CoordCube a, CoordCube b) {
        requireCompatible(a, b, "hammingDistance");
        Coord[] coordsA = a.coords();
        Coord[] coordsB = b.coords();
        int distance = 0;
        for (int i = 0; i < coordsA.length; i++) {
            if (!coordsA[i].equals(coordsB[i])) {
                distance++;
            }
        }
        return distance;
    }

    /**
     * The axis-wise Hamming distance: the per-dimension character differences,
     * in dimension order.
     *
     * @throws IllegalArgumentException when the cubes do not share
     *         {@code N}, {@code D} and {@code R}
     */
    public static int[] hammingDistanceAxes(CoordCube a, CoordCube b) {
        requireCompatible(a, b, "hammingDistanceAxes");
        Coord[] coordsA = a.coords();
        Coord[] coordsB = b.coords();
        int dimensions = a.ndim();
        int resolution = a.resolution();
        int[] axes = new int[dimensions];
        for (int dim = 0; dim < dimensions; dim++) {
            int start = dim * resolution;
            int difference = 0;
            for (int i = 0; i < resolution; i++) {
                if (!coordsA[start + i].equals(coordsB[start + i])) {
                    difference++;
                }
            }
            axes[dim] = difference;
        }
        return axes;
    }

    /**
     * The normalised Euclidean distance approximation in
     * {@code [0, sqrt(D)]}: each dimension value is normalised to
     * {@code [0, 1]} before the distance is accumulated, and the square root
     * uses {@link #sqrtApprox(double)}.
     *
     * <p>Dimension values are widened to {@code double} with unsigned 64-bit
     * semantics, as the reference conversions do, so a wrapped value with
     * bit 63 set keeps its magnitude instead of flipping sign.
     *
     * @throws IllegalArgumentException when the cubes do not share
     *         {@code N}, {@code D} and {@code R}
     */
    public static double euclideanDistanceApprox(CoordCube a, CoordCube b) {
        requireCompatible(a, b, "euclideanDistanceApprox");
        double maxValue = unsignedToDouble(dimensionMaxValue(a.resolution()));
        double sumOfSquares = 0.0;
        for (int dim = 0; dim < a.ndim(); dim++) {
            double valueA = unsignedToDouble(dimensionValue(a, dim));
            double valueB = unsignedToDouble(dimensionValue(b, dim));
            double difference = (valueA - valueB) / maxValue;
            sumOfSquares += difference * difference;
        }
        return sqrtApprox(sumOfSquares);
    }

    /**
     * The Manhattan (L1) distance: the sum of the per-dimension absolute
     * differences of the little-endian base-11172 dimension values.
     *
     * <p>The absolute difference uses unsigned 64-bit comparison and the sum
     * wraps like the unsigned reference accumulator, so the bit pattern
     * matches the C++ {@code std::uint64_t} and Rust {@code u64} results.
     *
     * @throws IllegalArgumentException when the cubes do not share
     *         {@code N}, {@code D} and {@code R}
     */
    public static long manhattanDistance(CoordCube a, CoordCube b) {
        requireCompatible(a, b, "manhattanDistance");
        long sum = 0L;
        for (int dim = 0; dim < a.ndim(); dim++) {
            long valueA = dimensionValue(a, dim);
            long valueB = dimensionValue(b, dim);
            sum += Long.compareUnsigned(valueA, valueB) >= 0 ? valueA - valueB : valueB - valueA;
        }
        return sum;
    }

    /**
     * The double nearest to the unsigned 64-bit interpretation of
     * {@code value}, rounding half to even, mirroring a C++
     * {@code static_cast<double>} of a {@code std::uint64_t} and a Rust
     * {@code u64 as f64} conversion.
     *
     * <p>Java has no unsigned conversion, and the signed widening of
     * {@code value} would move every bit pattern with bit 63 set down by 2^64,
     * turning a large positive reference value into a negative one.
     */
    private static double unsignedToDouble(long value) {
        if (value >= 0) {
            return value;
        }
        // Values with bit 63 set are in [2^63, 2^64), where the double spacing
        // is 2^11. The kept part is a multiple of 2^11 below 2^63 and the sum
        // is exactly representable, so only the discarded 11 bits can round.
        long discarded = value & 0x7FFL;
        long kept = value & Long.MAX_VALUE & ~0x7FFL;
        double result = 0x1.0p63 + kept;
        if (discarded > 0x400L || (discarded == 0x400L && (value & 0x800L) != 0)) {
            result += 0x1.0p11;
        }
        return result;
    }

    /**
     * A Newton-Raphson square root approximation, mirroring the reference so
     * the Java result matches the C++ and Rust results bit for bit apart from
     * the language-neutral double rounding.
     *
     * @return {@code 0.0} for non-positive input
     */
    public static double sqrtApprox(double x) {
        if (x <= 0.0) {
            return 0.0;
        }
        double guess = x;
        for (int i = 0; i < 12; i++) {
            guess = (guess + x / guess) * 0.5;
        }
        return guess;
    }

    private static void requireCompatible(CoordCube a, CoordCube b, String operation) {
        if (a.totalCharacters() != b.totalCharacters()
                || a.ndim() != b.ndim()
                || a.resolution() != b.resolution()) {
            throw new IllegalArgumentException(
                    "SpatialOps::" + operation + ": cubes must share N, D and R (got N="
                            + a.totalCharacters() + ", D=" + a.ndim() + ", R=" + a.resolution()
                            + " and N=" + b.totalCharacters() + ", D=" + b.ndim() + ", R=" + b.resolution() + ")");
        }
    }
}
