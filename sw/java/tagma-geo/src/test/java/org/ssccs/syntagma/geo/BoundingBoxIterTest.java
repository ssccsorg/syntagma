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
import org.ssccs.syntagma.core.CoordPath;

/**
 * Translation of the {@code BoundingBoxIter} cases of the C++ test suite
 * {@code sw/cpp/tagma_geo/tests/test_spatial.cpp} ({@code test_bb_iter_*}),
 * extended with the Rust capacity cases of
 * {@code sw/rust/geo/src/spatial.rs} and with the Java saturation and
 * numeric-bound checks.
 */
class BoundingBoxIterTest {

    @Test
    void singleCharacter() {
        BoundingBoxIter box = new BoundingBoxIter(new int[][] {{3, 3}});
        assertEquals(1L, box.countPaths(), "bb single count");
        assertFalse(box.isEmpty(), "bb single not empty");
        int count = 0;
        for (CoordPath path : box) {
            assertEquals(3, path.coords()[0].index(), "bb single index");
            count++;
        }
        assertEquals(1, count, "bb single iteration");
    }

    @Test
    void twoCharacters() {
        BoundingBoxIter box = new BoundingBoxIter(new int[][] {{1, 2}, {3, 4}});
        assertEquals(4L, box.countPaths(), "bb two count");
        List<CoordPath> paths = collect(box);
        assertEquals(4, paths.size(), "bb two iteration");
        assertEquals(1, paths.get(0).coords()[0].index(), "bb two first row");
        assertEquals(3, paths.get(0).coords()[1].index(), "bb two first column");
        assertEquals(2, paths.get(3).coords()[0].index(), "bb two last row");
        assertEquals(4, paths.get(3).coords()[1].index(), "bb two last column");
    }

    @Test
    void maxRange() {
        BoundingBoxIter box = new BoundingBoxIter(new int[][] {{0, 11171}});
        assertEquals(11172L, box.countPaths(), "bb max count");
        assertEquals(11172, count(box), "bb max iteration");
    }

    @Test
    void emptyN0() {
        BoundingBoxIter box = new BoundingBoxIter(new int[0][]);
        assertTrue(box.isEmpty(), "bb n0 empty");
        assertEquals(0L, box.countPaths(), "bb n0 count paths");
        assertEquals(0, count(box), "bb n0 iteration");
        assertEquals(0, box.ndim(), "bb n0 ndim");
    }

    @Test
    void exhaustion() {
        BoundingBoxIter box = new BoundingBoxIter(new int[][] {{0, 1}});
        assertFalse(box.isEmpty(), "bb exhaustion not empty at start");
        box.next();
        box.next();
        assertTrue(box.isEmpty(), "bb exhaustion empty after two steps");
        assertFalse(box.hasNext(), "bb exhaustion has no next");
        assertThrows(NoSuchElementException.class, box::next);
    }

    @Test
    void invalidRanges() {
        assertThrows(IllegalArgumentException.class,
                () -> new BoundingBoxIter(new int[][] {{5, 3}}),
                "bb inverted range throws");
        assertThrows(IllegalArgumentException.class,
                () -> new BoundingBoxIter(new int[][] {{0, 11172}}),
                "bb out of bounds range throws");
        assertThrows(IllegalArgumentException.class,
                () -> new BoundingBoxIter(new int[][] {{0, Coord.N_VALID}}),
                "bb bound at N_VALID throws");
    }

    @Test
    void negativeAndMalformedBoundsRejected() {
        assertThrows(IllegalArgumentException.class,
                () -> new BoundingBoxIter(new int[][] {{-1, 5}}),
                "bb negative min throws");
        assertThrows(IllegalArgumentException.class,
                () -> new BoundingBoxIter(new int[][] {{0, 1}, null}),
                "bb null range throws");
        assertThrows(IllegalArgumentException.class,
                () -> new BoundingBoxIter(new int[][] {{0, 1, 2}}),
                "bb three-element range throws");
    }

    @Test
    void countPaths() {
        BoundingBoxIter box = new BoundingBoxIter(new int[][] {{0, 2}, {0, 3}});
        assertEquals(12L, box.countPaths(), "bb count 3x4");
    }

    @Test
    void countPathsMultiDimensional() {
        BoundingBoxIter box = new BoundingBoxIter(new int[][] {{0, 1}, {0, 2}, {0, 3}});
        assertEquals(24L, box.countPaths(), "bb count 2x3x4");
    }

    @Test
    void countPathsLarge() {
        BoundingBoxIter box = new BoundingBoxIter(new int[][] {{0, 11171}, {0, 11171}});
        assertEquals(124813584L, box.countPaths(), "bb count 11172^2");
    }

    @Test
    void countPathsSaturatesOnOverflow() {
        int[][] ranges = new int[6][2];
        for (int i = 0; i < ranges.length; i++) {
            ranges[i] = new int[] {0, 11171};
        }
        BoundingBoxIter box = new BoundingBoxIter(ranges);
        assertEquals(Long.MAX_VALUE, box.countPaths(), "bb count saturates");
    }

    @Test
    void iteratorCopiesAreIndependent() {
        BoundingBoxIter box = new BoundingBoxIter(new int[][] {{0, 1}});
        assertEquals(2, count(box), "bb copy first pass");
        assertEquals(2, count(box), "bb copy second pass");
        assertEquals(0, box.next().coords()[0].index(), "bb copy consumed first path");
        assertEquals(1, count(box), "bb copy resumes at the current position");
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
