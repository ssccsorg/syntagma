package org.ssccs.syntagma.geo;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.Test;
import org.ssccs.syntagma.core.Coord;
import org.ssccs.syntagma.core.CoordCube;
import org.ssccs.syntagma.core.CoordPath;

/**
 * Translation of the Rust integration suite
 * {@code sw/rust/geo/tests/spatial_window.rs}: the bounded per-character
 * domain of {@code SpatialOps::proximityBounded}. Region generation is clamped
 * to a caller-defined domain {@code [0, domain)}; a store whose keys occupy a
 * narrower domain than the full {@code Coord} index space, such as a
 * byte-space store over {@code [0, 256)}, must bound generation, otherwise a
 * radius that crosses the domain edge wraps onto low index values (issue #59).
 */
class ProximityBoundedTest {

    private static final int BYTE_DOMAIN = 256;

    /**
     * A radius that crosses the domain edge must clamp at {@code domain - 1}
     * and must not wrap onto low index values.
     */
    @Test
    void clampsAtDomainEdge() {
        CoordCube cube = cubeAt(254);
        assertEquals(49L, SpatialOps.proximityBounded(cube, 5, BYTE_DOMAIN).countPaths(),
                "center 254 with radius 5 covers bytes 249..=255");
        assertEquals(49, assertWithinDomain(SpatialOps.proximityBounded(cube, 5, BYTE_DOMAIN), BYTE_DOMAIN));
    }

    /**
     * A radius that crosses the bottom of the domain clamps at byte 0.
     */
    @Test
    void clampsAtDomainFloor() {
        CoordCube cube = cubeAt(5);
        assertEquals(121, assertWithinDomain(SpatialOps.proximityBounded(cube, 5, BYTE_DOMAIN), BYTE_DOMAIN),
                "center 5 with radius 5 covers bytes 0..=10");
    }

    /**
     * Near the middle of the domain, bounded generation agrees with the
     * full-space default.
     */
    @Test
    void midDomainMatchesFullSpace() {
        CoordCube cube = cubeAt(136);
        BoundingBoxIter bounded = SpatialOps.proximityBounded(cube, 1, BYTE_DOMAIN);
        assertEquals(9L, bounded.countPaths(), "bounded mid count");
        assertEquals(9, assertWithinDomain(bounded, BYTE_DOMAIN));
        assertEquals(9L, SpatialOps.proximity(cube, 1).countPaths(), "full-space mid count");
    }

    /**
     * The full-space default clamps at the top of the {@code Coord} index
     * space.
     */
    @Test
    void fullSpaceClampsAtFullDomainEdge() {
        CoordCube cube = cubeAt(11171);
        assertEquals(16, assertWithinDomain(SpatialOps.proximity(cube, 3), Coord.N_VALID),
                "center 11171 with radius 3 covers 11168..=11171");
    }

    /**
     * A center at or above the domain is a caller error and is rejected
     * instead of silently wrapping.
     */
    @Test
    void centerAtOrAboveDomainRejected() {
        CoordCube cube = cubeAt(300);
        IllegalArgumentException error = assertThrows(IllegalArgumentException.class,
                () -> SpatialOps.proximityBounded(cube, 1, BYTE_DOMAIN));
        assertTrue(error.getMessage().contains("at or above domain " + BYTE_DOMAIN), error.getMessage());
    }

    /**
     * A zero domain is invalid.
     */
    @Test
    void zeroDomainRejected() {
        CoordCube cube = cubeAt(100);
        IllegalArgumentException error = assertThrows(IllegalArgumentException.class,
                () -> SpatialOps.proximityBounded(cube, 1, 0));
        assertTrue(error.getMessage().contains("domain 0 must be in"), error.getMessage());
    }

    /**
     * A domain larger than the {@code Coord} index space is invalid.
     */
    @Test
    void domainAboveValidRangeRejected() {
        CoordCube cube = cubeAt(100);
        IllegalArgumentException error = assertThrows(IllegalArgumentException.class,
                () -> SpatialOps.proximityBounded(cube, 1, Coord.N_VALID + 1));
        assertTrue(error.getMessage().contains("must be in"), error.getMessage());
        // The one-past domain is rejected while the full domain is accepted.
        assertEquals(1L, SpatialOps.proximityBounded(cube, 0, Coord.N_VALID).countPaths());
    }

    private static int assertWithinDomain(Iterable<CoordPath> paths, int domain) {
        int count = 0;
        for (CoordPath path : paths) {
            for (Coord coord : path.coords()) {
                assertTrue(coord.index() < domain, "path coordinate above domain");
            }
            count++;
        }
        return count;
    }

    private static CoordCube cubeAt(int value) {
        Coord coord = Coord.fromIndex(value).orElseThrow();
        return CoordCube.fromPath(2, 1, CoordPath.fromArray(coord, coord));
    }
}
