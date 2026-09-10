package org.ssccs.syntagma.bench;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

import java.util.List;

import org.junit.jupiter.api.Test;
import org.ssccs.syntagma.core.Coord;
import org.ssccs.syntagma.core.CoordPath;

/**
 * Cases for {@link BenchInputs}: the path formulas of the C++ helpers
 * {@code paths_2d} and {@code paths_3d} in {@code sw/cpp/bench/bench.cpp},
 * on the boundary where the first coordinate wraps.
 */
class BenchInputsTest {

    @Test
    void paths2dWalkTheReferenceFormula() {
        List<CoordPath> paths = BenchInputs.paths2d(3);

        assertEquals(3, paths.size(), "count");
        assertEquals(CoordPath.fromArray(BenchInputs.coord(0), BenchInputs.coord(0)), paths.get(0));
        assertEquals(CoordPath.fromArray(BenchInputs.coord(1), BenchInputs.coord(0)), paths.get(1));
        assertEquals(CoordPath.fromArray(BenchInputs.coord(2), BenchInputs.coord(0)), paths.get(2));
    }

    @Test
    void paths2dWrapTheFirstCoordinateIntoTheSecond() {
        List<CoordPath> paths = BenchInputs.paths2d(Coord.N_VALID + 1);

        assertEquals(CoordPath.fromArray(BenchInputs.coord(Coord.N_VALID - 1), BenchInputs.coord(0)),
                paths.get(Coord.N_VALID - 1), "last distinct path");
        assertEquals(CoordPath.fromArray(BenchInputs.coord(0), BenchInputs.coord(1)),
                paths.get(Coord.N_VALID), "wrapped path");
    }

    @Test
    void paths3dAppendTheZeroCoordinate() {
        List<CoordPath> paths = BenchInputs.paths3d(2);

        assertEquals(CoordPath.fromArray(BenchInputs.coord(0), BenchInputs.coord(0), BenchInputs.coord(0)),
                paths.get(0), "first path");
        assertEquals(3, paths.get(0).length(), "path length");
        assertEquals(CoordPath.fromArray(BenchInputs.coord(1), BenchInputs.coord(0), BenchInputs.coord(0)),
                paths.get(1), "second path");
    }

    @Test
    void coordRejectsAnIndexOutsideTheLattice() {
        assertThrows(IllegalArgumentException.class, () -> BenchInputs.coord(Coord.N_VALID));
        assertThrows(IllegalArgumentException.class, () -> BenchInputs.coord(-1));
    }

    @Test
    void rejectsNegativeCounts() {
        assertThrows(IllegalArgumentException.class, () -> BenchInputs.paths2d(-1));
        assertThrows(IllegalArgumentException.class, () -> BenchInputs.paths3d(-1));
    }
}
