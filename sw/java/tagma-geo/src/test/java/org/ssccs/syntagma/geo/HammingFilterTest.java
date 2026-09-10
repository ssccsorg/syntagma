package org.ssccs.syntagma.geo;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.ArrayList;
import java.util.List;
import java.util.NoSuchElementException;

import org.junit.jupiter.api.Test;
import org.ssccs.syntagma.core.Coord;
import org.ssccs.syntagma.core.CoordCube;
import org.ssccs.syntagma.core.CoordPath;

/**
 * Translation of the {@code HammingFilter} cases of the C++ test suite
 * {@code sw/cpp/tagma_geo/tests/test_spatial.cpp}
 * ({@code test_hamming_filter_direct}) and of the Rust
 * {@code hamming_filter_direct} case in {@code sw/rust/geo/src/spatial.rs},
 * extended with the Java length check.
 */
class HammingFilterTest {

    @Test
    void filtersWithinHammingRadius() {
        BoundingBoxIter box = new BoundingBoxIter(new int[][] {{0, 2}, {0, 2}});
        CoordPath center = cube(2, 1, 1, 1).asPath();
        HammingFilter filter = new HammingFilter(box, center, 1);
        // In a 3x3 box around center (1,1): center + 4 axis neighbors.
        assertEquals(5, count(filter), "hamming filter direct count");
    }

    @Test
    void filtersOnlyCenterAtRadiusZero() {
        BoundingBoxIter box = new BoundingBoxIter(new int[][] {{4, 6}, {4, 6}});
        CoordPath center = cube(2, 1, 5, 5).asPath();
        HammingFilter filter = new HammingFilter(box, center, 0);
        List<CoordPath> paths = collect(filter);
        assertEquals(1, paths.size(), "hamming filter r0 count");
        assertEquals(5, paths.get(0).coords()[0].index(), "hamming filter r0 index");
    }

    @Test
    void emptyWhenNoCandidateIsWithinRadius() {
        BoundingBoxIter box = new BoundingBoxIter(new int[][] {{0, 0}, {0, 0}});
        CoordPath center = cube(2, 1, 5, 5).asPath();
        HammingFilter filter = new HammingFilter(box, center, 0);
        assertFalse(filter.hasNext(), "hamming filter empty");
        assertEquals(0, count(filter), "hamming filter empty iteration");
        assertThrows(NoSuchElementException.class, filter::next);
    }

    @Test
    void iteratorCopiesAreIndependent() {
        BoundingBoxIter box = new BoundingBoxIter(new int[][] {{0, 2}, {0, 2}});
        CoordPath center = cube(2, 1, 1, 1).asPath();
        HammingFilter filter = new HammingFilter(box, center, 1);
        assertEquals(5, count(filter), "hamming filter copy first pass");
        assertEquals(5, count(filter), "hamming filter copy second pass");
    }

    @Test
    void rejectsCenterLengthMismatch() {
        BoundingBoxIter box = new BoundingBoxIter(new int[][] {{0, 2}, {0, 2}});
        CoordPath center = CoordPath.fromArray(coord(1));
        IllegalArgumentException error = assertThrows(IllegalArgumentException.class,
                () -> new HammingFilter(box, center, 1));
        assertTrue(error.getMessage().contains("center length"), error.getMessage());
    }

    @Test
    void rejectsNegativeMaxDistance() {
        BoundingBoxIter box = new BoundingBoxIter(new int[][] {{0, 2}, {0, 2}});
        CoordPath center = cube(2, 1, 1, 1).asPath();
        assertThrows(IllegalArgumentException.class, () -> new HammingFilter(box, center, -1));
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

    private static List<CoordPath> collect(Iterable<CoordPath> paths) {
        List<CoordPath> collected = new ArrayList<>();
        for (CoordPath path : paths) {
            collected.add(path);
        }
        return collected;
    }
}
