package org.ssccs.syntagma.core;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.Test;

/**
 * Translation of the C++ test suite {@code sw/cpp/tagma_core/tests/test_coord_cube.cpp}:
 * construction, axis decomposition, coordinate access, equality, and display.
 */
class CoordCubeTest {

    private static Coord coord(int index) {
        return Coord.fromIndex(index).orElseThrow();
    }

    /**
     * The reference example: N = 6, D = 3, R = 2, coords 0..5.
     */
    private static CoordCube exampleCube() {
        Coord[] coords = {coord(0), coord(1), coord(2), coord(3), coord(4), coord(5)};
        return CoordCube.fromPath(3, 2, CoordPath.fromArray(coords));
    }

    @Test
    void dimensionsAndResolution() {
        CoordCube cube = exampleCube();
        assertEquals(3, cube.ndim(), "cube ndim");
        assertEquals(2, cube.resolution(), "cube resolution");
        assertEquals(6, cube.totalCharacters(), "cube total characters");
    }

    @Test
    void axisDecomposition() {
        CoordCube cube = exampleCube();
        CoordPath axis0 = cube.axis(0);
        assertEquals(0, axis0.coords()[0].index(), "axis 0 char 0");
        assertEquals(1, axis0.coords()[1].index(), "axis 0 char 1");
        assertEquals(2, cube.axis(1).coords()[0].index(), "axis 1 char 0");
        assertEquals(3, cube.axis(1).coords()[1].index(), "axis 1 char 1");
        assertEquals(4, cube.axis(2).coords()[0].index(), "axis 2 char 0");
        assertEquals(5, cube.axis(2).coords()[1].index(), "axis 2 char 1");
    }

    @Test
    void coordinateAccess() {
        CoordCube cube = exampleCube();
        assertEquals(0, cube.coordAt(0, 0).index(), "coord at origin");
        assertEquals(5, cube.coordAt(2, 1).index(), "coord at last");
        assertEquals(6, cube.coords().length, "cube coords size");
    }

    @Test
    void pathRoundTrip() {
        CoordCube cube = exampleCube();
        assertArrayEquals(cube.coords(), cube.asPath().coords(), "as_path round trip");
        assertEquals(cube.asPath(), cube.intoPath(), "into_path round trip");
    }

    @Test
    void equality() {
        CoordCube same = exampleCube();
        assertEquals(exampleCube(), same, "cube equality");

        Coord[] otherCoords = {coord(5), coord(4), coord(3), coord(2), coord(1), coord(0)};
        CoordCube other = CoordCube.fromPath(3, 2, CoordPath.fromArray(otherCoords));
        assertNotEquals(exampleCube(), other, "cube inequality");
    }

    @Test
    void display() {
        assertEquals("CoordCube<6, 3, 2>[(가, 각) | (갂, 갃) | (간, 갅)]", exampleCube().toString());
        CoordPath path3 = CoordPath.fromArray(coord(0), coord(1), coord(2));
        assertEquals("CoordPath<3>(가, 각, 갂)", path3.toString());
    }

    @Test
    void invalidDimensionsThrows() {
        Coord[] coords = {coord(0), coord(0), coord(0), coord(0)};
        IllegalArgumentException error = assertThrows(IllegalArgumentException.class,
                () -> CoordCube.fromPath(3, 1, CoordPath.fromArray(coords)));
        assertTrue(error.getMessage().contains("must equal D*R"), error.getMessage());
    }

    @Test
    void axisOutOfRangeThrows() {
        CoordCube cube = exampleCube();
        assertThrows(IllegalArgumentException.class, () -> cube.axis(3));
        assertThrows(IllegalArgumentException.class, () -> cube.coordAt(3, 0));
        assertThrows(IllegalArgumentException.class, () -> cube.coordAt(0, 2));
    }
}
