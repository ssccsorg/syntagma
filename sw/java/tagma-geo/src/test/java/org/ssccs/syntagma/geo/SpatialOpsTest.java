package org.ssccs.syntagma.geo;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.Test;
import org.ssccs.syntagma.core.Coord;
import org.ssccs.syntagma.core.CoordCube;
import org.ssccs.syntagma.core.CoordPath;

/**
 * Translation of the {@code SpatialOps} and {@code DistanceMetrics} cases of
 * the C++ test suite {@code sw/cpp/tagma_geo/tests/test_spatial.cpp}
 * ({@code test_cube_bounding_box_basic}, {@code test_cube_proximity},
 * {@code test_proximity_bounded}, {@code test_hamming},
 * {@code test_hamming_axes}, {@code test_euclidean}, {@code test_manhattan},
 * {@code test_proximity_hamming}), extended with the multi-character cases of
 * the Rust module tests in {@code sw/rust/geo/src/spatial.rs}.
 *
 * <p>The domain-window contract of {@code proximityBounded} is covered by
 * {@link ProximityBoundedTest}, the translation of the Rust integration suite
 * {@code sw/rust/geo/tests/spatial_window.rs}.
 */
class SpatialOpsTest {

    @Test
    void cubeBoundingBoxBasic() {
        CoordCube cube = cube(2, 1, 0, 0);
        BoundingBoxIter box = SpatialOps.boundingBox(cube, new int[][] {{0, 1}, {2, 3}});
        assertEquals(4L, box.countPaths(), "cube bb count");
        assertEquals(4, count(box), "cube bb iteration");
    }

    @Test
    void cubeBoundingBoxMultiCharacter() {
        CoordCube cube = cube(2, 2, 0, 0, 0, 0);
        BoundingBoxIter box = SpatialOps.boundingBox(cube, new int[][] {{0, 1}, {0, 0}, {0, 1}, {0, 0}});
        // Dimension 0 contributes two paths, dimension 1 contributes two.
        assertEquals(4L, box.countPaths(), "cube bb multi count");
        assertEquals(4, count(box), "cube bb multi iteration");
    }

    @Test
    void cubeBoundingBoxRejectsCharacterCountMismatch() {
        CoordCube cube = cube(2, 1, 0, 0);
        IllegalArgumentException error = assertThrows(IllegalArgumentException.class,
                () -> SpatialOps.boundingBox(cube, new int[][] {{0, 1}}));
        assertTrue(error.getMessage().contains("must equal the cube character count"), error.getMessage());
    }

    @Test
    void cubeProximity() {
        CoordCube cube = cube(2, 1, 5, 10);

        assertEquals(1L, SpatialOps.proximity(cube, 0).countPaths(), "proximity r0 count");
        assertEquals(9L, SpatialOps.proximity(cube, 1).countPaths(), "proximity r1 count");

        // Clamp to bounds: cube at index 0.
        CoordCube edge = cube(2, 1, 0, 11171);
        assertEquals(4L * 4L, SpatialOps.proximity(edge, 3).countPaths(), "proximity clamp count");
    }

    @Test
    void cubeProximityMultiCharacter() {
        CoordCube cube = cube(2, 2, 5, 5, 5, 5);
        // 4 characters, each range width 3: 3^4.
        assertEquals(81L, SpatialOps.proximity(cube, 1).countPaths(), "proximity multi count");
    }

    @Test
    void proximityBounded() {
        // Edge clamp inside a byte-size domain: center 254, radius 5, domain 256.
        CoordCube edgeCube = cube(2, 1, 254, 254);
        BoundingBoxIter bounded = SpatialOps.proximityBounded(edgeCube, 5, 256);
        assertEquals(49L, bounded.countPaths(), "proximity bounded edge count");
        int seen = 0;
        boolean inDomain = true;
        for (CoordPath path : bounded) {
            for (Coord coord : path.coords()) {
                if (coord.index() >= 256) {
                    inDomain = false;
                }
            }
            seen++;
        }
        assertEquals(49, seen, "proximity bounded edge iteration");
        assertTrue(inDomain, "proximity bounded no wrap below domain");

        // Middle of the domain agrees with the full-domain default.
        CoordCube midCube = cube(2, 1, 136, 136);
        assertEquals(9L, SpatialOps.proximityBounded(midCube, 1, 256).countPaths(),
                "proximity bounded mid count");
        assertEquals(9L, SpatialOps.proximity(midCube, 1).countPaths(), "proximity full mid count");

        // The full-domain default clamps at the top of the Coord index space.
        CoordCube top = cube(2, 1, 11171, 11171);
        assertEquals(16L, SpatialOps.proximity(top, 3).countPaths(), "proximity full top clamp");

        assertThrows(IllegalArgumentException.class,
                () -> SpatialOps.proximityBounded(midCube, 1, 0),
                "proximity bounded zero domain throws");

        CoordCube high = cube(2, 1, 300, 300);
        assertThrows(IllegalArgumentException.class,
                () -> SpatialOps.proximityBounded(high, 1, 256),
                "proximity bounded out-of-domain center throws");
    }

    @Test
    void proximityBoundedRejectsDomainAboveValidRange() {
        CoordCube cube = cube(2, 1, 100, 100);
        IllegalArgumentException error = assertThrows(IllegalArgumentException.class,
                () -> SpatialOps.proximityBounded(cube, 1, Coord.N_VALID + 1));
        assertTrue(error.getMessage().contains("must be in"), error.getMessage());
    }

    @Test
    void proximityBoundedRejectsNegativeRadius() {
        CoordCube cube = cube(2, 1, 100, 100);
        assertThrows(IllegalArgumentException.class, () -> SpatialOps.proximityBounded(cube, -1, 256));
    }

    @Test
    void hamming() {
        CoordCube a = cube(2, 1, 0, 0);
        CoordCube b = cube(2, 1, 0, 5);
        assertEquals(1, SpatialOps.hammingDistance(a, b), "hamming differ one");
        assertEquals(0, SpatialOps.hammingDistance(a, a), "hamming identical");

        CoordCube allDiff = cube(2, 1, 3, 4);
        assertEquals(2, SpatialOps.hammingDistance(a, allDiff), "hamming all differ");
    }

    @Test
    void hammingAxes() {
        // dim0 = coords[0..1], dim1 = coords[2..3].
        CoordCube a = cube(2, 2, 0, 0, 0, 0);
        CoordCube b = cube(2, 2, 0, 1, 2, 3);
        int[] axes = SpatialOps.hammingDistanceAxes(a, b);
        assertArrayEquals(new int[] {1, 2}, axes, "hamming axes values");
    }

    @Test
    void euclidean() {
        CoordCube a = cube(2, 1, 0, 0);
        assertEquals(0.0, SpatialOps.euclideanDistanceApprox(a, a), 1e-9, "euclidean identical");

        CoordCube b = cube(2, 1, 11171, 0);
        assertEquals(1.0, SpatialOps.euclideanDistanceApprox(a, b), 1e-9, "euclidean max in one dim");
    }

    @Test
    void euclideanMultiCharacter() {
        // A single dimension with two characters: half of the dimension range
        // is close to a normalised distance of 0.5.
        CoordCube a = cube(1, 2, 0, 0);
        CoordCube b = cube(1, 2, 0, 5586);
        assertEquals(0.5, SpatialOps.euclideanDistanceApprox(a, b), 0.01, "euclidean half dimension");
    }

    @Test
    void manhattan() {
        CoordCube a = cube(2, 1, 0, 0);
        assertEquals(0L, SpatialOps.manhattanDistance(a, a), "manhattan identical");

        CoordCube b = cube(2, 1, 5, 0);
        assertEquals(5L, SpatialOps.manhattanDistance(a, b), "manhattan different");

        CoordCube c = cube(2, 2, 0, 0, 0, 0);
        CoordCube d = cube(2, 2, 1, 0, 0, 0);
        assertEquals(1L, SpatialOps.manhattanDistance(c, d), "manhattan multi character");
    }

    @Test
    void manhattanMultiCharacter() {
        CoordCube a = cube(1, 2, 0, 0);
        CoordCube b = cube(1, 2, 10, 20);
        assertEquals(10L + 20L * Coord.N_VALID, SpatialOps.manhattanDistance(a, b),
                "manhattan base-11172 value");
    }

    @Test
    void proximityHamming() {
        CoordCube cube = cube(2, 1, 5, 10);

        int count0 = 0;
        for (CoordPath path : SpatialOps.proximityHamming(cube, 0)) {
            assertEquals(cube.asPath(), path, "proximity hamming r0 only center");
            count0++;
        }
        assertEquals(1, count0, "proximity hamming r0 count");

        // Center + 4 axis neighbors at Hamming distance 1 within radius 1 box.
        assertEquals(5, count(SpatialOps.proximityHamming(cube, 1)), "proximity hamming r1 count");
    }

    @Test
    void dimensionValue() {
        CoordCube cube = cube(2, 1, 3, 4);
        assertEquals(3L, SpatialOps.dimensionValue(cube, 0), "dimension value dim 0");
        assertEquals(4L, SpatialOps.dimensionValue(cube, 1), "dimension value dim 1");

        CoordCube wide = cube(1, 2, 10, 20);
        assertEquals(10L + 20L * Coord.N_VALID, SpatialOps.dimensionValue(wide, 0),
                "dimension value little-endian base 11172");

        assertThrows(IllegalArgumentException.class, () -> SpatialOps.dimensionValue(cube, 2),
                "dimension value dim out of range");
        assertThrows(IllegalArgumentException.class, () -> SpatialOps.dimensionValue(cube, -1),
                "dimension value negative dim");
    }

    @Test
    void dimensionMaxValue() {
        assertEquals(0L, SpatialOps.dimensionMaxValue(0), "dimension max R0");
        assertEquals(11171L, SpatialOps.dimensionMaxValue(1), "dimension max R1");
        assertEquals(124813583L, SpatialOps.dimensionMaxValue(2), "dimension max R2");
        assertThrows(IllegalArgumentException.class, () -> SpatialOps.dimensionMaxValue(-1));
    }

    @Test
    void sqrtApprox() {
        assertEquals(0.0, SpatialOps.sqrtApprox(0.0), "sqrt zero");
        assertEquals(0.0, SpatialOps.sqrtApprox(-4.0), "sqrt negative");
        assertEquals(2.0, SpatialOps.sqrtApprox(4.0), 1e-9, "sqrt four");
        assertEquals(3.0, SpatialOps.sqrtApprox(9.0), 1e-9, "sqrt nine");
    }

    @Test
    void distanceRejectsCubesWithoutSharedShape() {
        CoordCube two = cube(2, 1, 0, 0);
        CoordCube four = cube(2, 2, 0, 0, 0, 0);
        CoordCube alsoFour = cube(4, 1, 0, 0, 0, 0);

        assertThrows(IllegalArgumentException.class, () -> SpatialOps.hammingDistance(two, four));
        assertThrows(IllegalArgumentException.class, () -> SpatialOps.hammingDistanceAxes(two, four));
        assertThrows(IllegalArgumentException.class, () -> SpatialOps.euclideanDistanceApprox(two, four));
        assertThrows(IllegalArgumentException.class, () -> SpatialOps.manhattanDistance(two, four));
        assertThrows(IllegalArgumentException.class, () -> SpatialOps.hammingDistance(four, alsoFour));
    }

    private static Coord coord(int index) {
        return Coord.fromIndex(index).orElseThrow();
    }

    private static CoordCube cube(int dimensions, int resolution, int... indices) {
        Coord[] coords = new Coord[indices.length];
        for (int i = 0; i < indices.length; i++) {
            coords[i] = coord(indices[i]);
        }
        return CoordCube.fromPath(dimensions, resolution, CoordPath.fromArray(coords));
    }

    private static int count(Iterable<CoordPath> paths) {
        int count = 0;
        for (CoordPath ignored : paths) {
            count++;
        }
        return count;
    }
}
