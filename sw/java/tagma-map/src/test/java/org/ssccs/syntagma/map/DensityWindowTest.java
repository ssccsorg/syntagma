package org.ssccs.syntagma.map;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import java.util.List;

import org.junit.jupiter.api.Test;
import org.ssccs.syntagma.core.Coord;
import org.ssccs.syntagma.core.CoordPath;

/**
 * Translation of the Rust integration suite
 * {@code sw/rust/map/tests/density_window.rs}: the density scenarios behind the
 * {@code Spatial/map_proximity} benchmark record under the explicit map
 * byte-space domain. The map key space is one byte per character, so
 * storage-backed spatial queries operate in the per-character domain
 * {@code [0, 256)}: region generation is bounded to that domain, centers and
 * range bounds outside it are rejected, and a radius that crosses the domain
 * edge clamps instead of wrapping onto low byte values (issue #59).
 */
class DensityWindowTest {

    private static final CoordPath CENTER_136 = path(136, 136);

    /**
     * The dense benchmark fill mapped onto the byte domain: a 100x100 box over
     * bytes {@code 86..=186} with the query center at byte 136. Every generated
     * neighbor lies inside the filled box, so the found counts match the record
     * (9 hits at radius 1, 25 hits at radius 2) without coordinate wrapping.
     */
    @Test
    void denseScenario100x100FoundCounts() {
        CoordMapN map = new CoordMapN(2);
        for (int x = 86; x <= 186; x++) {
            for (int y = 86; y <= 186; y++) {
                map.insertByCoordKey(CoordKey.fromCoordPath(path(x, y)), bytes("v"));
            }
        }

        List<CoordCubeMap.Hit> radius1 = CoordCubeMap.proximity(map, CENTER_136, 1, 2, 1);
        assertEquals(9, radius1.size(), "dense r=1: all 9 generated neighbors present");

        List<CoordCubeMap.Hit> radius2 = CoordCubeMap.proximity(map, CENTER_136, 2, 2, 1);
        assertEquals(25, radius2.size(), "dense r=2: all 25 generated neighbors present");
    }

    /**
     * The sparse benchmark fill mapped onto the byte domain: nine scattered
     * bytes {@code {86, 136, 186}} in both axes. A radius-1 query centered at
     * byte 136 covers bytes {@code 135..=137}, so only the center entry
     * (136, 136) is found (issue #59).
     */
    @Test
    void sparseScenarioNineScatteredEntriesFoundCount() {
        CoordMapN map = new CoordMapN(2);
        int[] scatter = {86, 136, 186};
        for (int x : scatter) {
            for (int y : scatter) {
                map.insertByCoordKey(new CoordKey(new byte[] {(byte) x, (byte) y}), bytes("v"));
            }
        }

        List<CoordCubeMap.Hit> results = CoordCubeMap.proximity(map, CENTER_136, 1, 2, 1);
        assertEquals(1, results.size(), "byte-space window: only (136, 136) lies in 135..=137");
        assertArrayEquals(bytes("v"), results.get(0).value(), "sparse center value");
    }

    /** The empty benchmark scenario: no entries, so the query returns nothing. */
    @Test
    void emptyScenarioFoundCount() {
        CoordMapN map = new CoordMapN(2);
        assertTrue(CoordCubeMap.proximity(map, CENTER_136, 1, 2, 1).isEmpty(),
                "empty store has no neighbors");
    }

    /**
     * A radius that crosses the top of the byte domain clamps at byte 255.
     * Entries in bytes {@code 0..=5} are not reached through wrapping.
     */
    @Test
    void proximityDoesNotWrapAcrossByteDomainEdge() {
        CoordMapN map = new CoordMapN(2);
        for (int x = 0; x <= 5; x++) {
            for (int y = 0; y <= 5; y++) {
                map.insertByCoordKey(CoordKey.fromCoordPath(path(x, y)), bytes("low"));
            }
        }
        for (int x = 250; x <= 255; x++) {
            for (int y = 250; y <= 255; y++) {
                map.insertByCoordKey(CoordKey.fromCoordPath(path(x, y)), bytes("high"));
            }
        }

        List<CoordCubeMap.Hit> results = CoordCubeMap.proximity(map, path(255, 255), 2, 2, 1);
        assertEquals(9, results.size(), "radius 2 from byte 255 covers bytes 253..=255 only");
        for (CoordCubeMap.Hit hit : results) {
            for (Coord coord : hit.path().coords()) {
                assertTrue(coord.index() >= 253, "wrapped low-byte entry leaked into the result");
            }
        }
    }

    /**
     * A radius that reaches far beyond the byte domain is clamped to the domain
     * window, and the pre-sized result follows the bounded region rather than
     * the unbounded radius, whose reference formula {@code (2 * radius + 1) ^ N}
     * sizes the allocation in the tens of billions for this call.
     */
    @Test
    void hugeRadiusStaysBounded() {
        CoordMapN map = new CoordMapN(2);
        for (int x = 250; x <= 255; x++) {
            for (int y = 250; y <= 255; y++) {
                map.insertByCoordKey(CoordKey.fromCoordPath(path(x, y)), bytes("v"));
            }
        }

        // The window of a radius beyond the domain is the whole byte domain per
        // character, so every stored entry is found and no path leaves the
        // domain.
        List<CoordCubeMap.Hit> results = CoordCubeMap.proximity(map, path(255, 255), 50000, 2, 1);
        assertEquals(36, results.size(), "a radius beyond the domain covers the whole byte domain");
        for (CoordCubeMap.Hit hit : results) {
            for (Coord coord : hit.path().coords()) {
                assertTrue(coord.index() < CoordKey.BYTE_DOMAIN, "generated path stays in the domain");
            }
        }
    }

    /**
     * A center at or above the byte-space domain is a caller error and is
     * rejected instead of silently wrapping onto low bytes.
     */
    @Test
    void proximityCenterOutsideByteDomainRejected() {
        CoordMapN map = new CoordMapN(2);
        IllegalArgumentException error = assertThrows(IllegalArgumentException.class,
                () -> CoordCubeMap.proximity(map, path(300, 300), 1, 2, 1),
                "center above the byte domain rejected");
        assertTrue(error.getMessage().contains("outside the map byte-space domain"),
                error.getMessage());
    }

    /**
     * {@code fromCoordPath} cannot represent a character index at or above
     * {@link CoordKey#BYTE_DOMAIN} and is rejected instead of truncating.
     */
    @Test
    void fromCoordPathRejectsIndex256() {
        CoordPath path = path(256);
        IllegalArgumentException error = assertThrows(IllegalArgumentException.class,
                () -> CoordKey.fromCoordPath(path), "index 256 rejected");
        assertTrue(error.getMessage().contains("exceeds the byte-space domain"), error.getMessage());
        assertTrue(error.getMessage().contains("index 256"), error.getMessage());
    }

    /**
     * {@code boundingBoxRange} rejects range bounds at or above the byte domain
     * before any generation runs.
     */
    @Test
    void boundingBoxRangeOutsideByteDomainRejected() {
        CoordMapN map = new CoordMapN(2);
        IllegalArgumentException error = assertThrows(IllegalArgumentException.class,
                () -> CoordCubeMap.boundingBoxRange(map, new int[][] {{0, 255}, {0, 300}}),
                "range bound above the byte domain rejected");
        assertTrue(error.getMessage().contains("outside the map byte-space domain"),
                error.getMessage());
    }

    private static byte[] bytes(String text) {
        return text.getBytes(StandardCharsets.UTF_8);
    }

    private static CoordPath path(int... indices) {
        Coord[] coords = new Coord[indices.length];
        for (int i = 0; i < indices.length; i++) {
            coords[i] = Coord.fromIndex(indices[i]).orElseThrow();
        }
        return CoordPath.fromArray(coords);
    }
}
