package org.ssccs.syntagma.map;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.List;

import org.junit.jupiter.api.Test;
import org.ssccs.syntagma.core.Coord;
import org.ssccs.syntagma.core.CoordPath;

/**
 * Translation of the C++ test suite
 * {@code sw/cpp/tagma_map/tests/test_cube_map.cpp}: the spatial queries of
 * {@link CoordCubeMap} over {@code CoordMap2}, {@code CoordMapN} and
 * {@code DynCoordMap} ({@code test_map2_proximity}, {@code test_map2_bounding_box},
 * {@code test_mapn_proximity}, {@code test_mapn_empty},
 * {@code test_dynmap_spatial}, {@code test_byte_domain_edge},
 * {@code test_byte_domain_rejections}), with the membership checks of the Rust
 * suite {@code sw/rust/map/src/coord_cube_map.rs}. The dense, sparse and empty
 * benchmark scenarios of {@code sw/rust/map/tests/density_window.rs} are
 * translated by {@link DensityWindowTest}.
 */
class CoordCubeMapTest {

    @Test
    void map2Proximity() {
        CoordMap2 map = new CoordMap2();
        CoordPath center = path(5, 5);
        CoordPath nearby = path(5, 6);
        map.insertByCoordKey(new CoordKey(new byte[] {5, 5}), bytes("center"));
        map.insertByCoordKey(new CoordKey(new byte[] {5, 6}), bytes("nearby"));
        map.insertByCoordKey(new CoordKey(new byte[] {5, 20}), bytes("far"));

        List<CoordCubeMap.Hit> results = CoordCubeMap.proximity(map, center, 1, 2, 1);
        assertEquals(2, results.size(), "map2 proximity finds 2");
        List<CoordPath> found = pathsOf(results);
        assertTrue(found.contains(center), "map2 proximity finds the center");
        assertTrue(found.contains(nearby), "map2 proximity finds the neighbor");
        assertArrayEquals(bytes("center"), results.get(found.indexOf(center)).value(),
                "map2 proximity center value");
    }

    @Test
    void map2BoundingBox() {
        CoordMap2 map = new CoordMap2();
        map.insertByCoordKey(new CoordKey(new byte[] {5, 5}), bytes("v1"));
        map.insertByCoordKey(new CoordKey(new byte[] {5, 6}), bytes("v2"));
        map.insertByCoordKey(new CoordKey(new byte[] {10, 10}), bytes("v3"));

        List<CoordCubeMap.Hit> results = CoordCubeMap.boundingBoxRange(map,
                new int[][] {{4, 6}, {5, 7}});
        assertEquals(2, results.size(), "map2 bounding box finds 2");
    }

    @Test
    void mapNProximity() {
        CoordMapN map = new CoordMapN(3);
        CoordPath center = path(5, 5, 5);
        CoordPath nearby = path(5, 5, 6);
        map.insertByCoordKey(CoordKey.fromCoordPath(center), bytes("center"));
        map.insertByCoordKey(CoordKey.fromCoordPath(nearby), bytes("nearby"));
        map.insertByCoordKey(CoordKey.fromCoordPath(path(5, 5, 20)), bytes("far"));

        List<CoordCubeMap.Hit> results = CoordCubeMap.proximity(map, center, 1, 3, 1);
        assertEquals(2, results.size(), "mapn proximity finds 2");
        assertTrue(pathsOf(results).contains(nearby), "mapn proximity finds the neighbor");
    }

    @Test
    void emptyStores() {
        CoordMapN mapN = new CoordMapN(2);
        assertTrue(CoordCubeMap.proximity(mapN, path(5, 5), 1, 2, 1).isEmpty(),
                "mapn empty proximity");
        assertTrue(CoordCubeMap.boundingBoxRange(mapN, new int[][] {{0, 100}, {0, 100}}).isEmpty(),
                "mapn empty box");

        CoordMap2 map2 = new CoordMap2();
        assertTrue(CoordCubeMap.proximity(map2, path(5, 5), 1, 2, 1).isEmpty(),
                "map2 empty proximity");
        assertTrue(CoordCubeMap.boundingBoxRange(map2, new int[][] {{0, 100}, {0, 100}}).isEmpty(),
                "map2 empty box");

        DynCoordMap dynMap = new DynCoordMap();
        assertTrue(CoordCubeMap.proximity(dynMap, path(5, 5), 1, 2, 1).isEmpty(),
                "dynmap empty proximity");
        assertTrue(CoordCubeMap.boundingBoxRange(dynMap, new int[][] {{0, 100}, {0, 100}}).isEmpty(),
                "dynmap empty box");
    }

    @Test
    void dynMapSpatial() {
        DynCoordMap map = new DynCoordMap();
        map.insert("ab", bytes("v1"));
        map.insert("ac", bytes("v2"));
        map.insert("az", bytes("v3"));

        List<CoordCubeMap.Hit> proximity = CoordCubeMap.proximity(map, path(97, 98), 1, 2, 1);
        assertEquals(2, proximity.size(), "dynmap proximity finds 2");

        List<CoordCubeMap.Hit> box = CoordCubeMap.boundingBoxRange(map,
                new int[][] {{97, 99}, {98, 100}});
        assertEquals(2, box.size(), "dynmap bounding box finds 2");

        // "az" sits at (97, 122), so the box must cover index 122 to include
        // all three entries.
        List<CoordCubeMap.Hit> wide = CoordCubeMap.boundingBoxRange(map,
                new int[][] {{0, 200}, {0, 200}});
        assertEquals(3, wide.size(), "dynmap wide box finds 3");
    }

    @Test
    void byteDomainEdge() {
        CoordMapN map = new CoordMapN(2);
        // Low byte entries must not be reachable by wrapping from the top.
        for (int x = 0; x <= 5; x++) {
            for (int y = 0; y <= 5; y++) {
                map.insertByCoordKey(new CoordKey(new byte[] {(byte) x, (byte) y}), bytes("low"));
            }
        }
        for (int x = 250; x <= 255; x++) {
            for (int y = 250; y <= 255; y++) {
                map.insertByCoordKey(new CoordKey(new byte[] {(byte) x, (byte) y}), bytes("high"));
            }
        }

        List<CoordCubeMap.Hit> results = CoordCubeMap.proximity(map, path(255, 255), 2, 2, 1);
        assertEquals(9, results.size(), "byte domain edge proximity count");
        boolean inDomain = true;
        for (CoordCubeMap.Hit hit : results) {
            for (Coord coord : hit.path().coords()) {
                if (coord.index() < 253) {
                    inDomain = false;
                }
            }
        }
        assertTrue(inDomain, "byte domain edge no wrap");
    }

    @Test
    void byteDomainRejections() {
        CoordMapN map = new CoordMapN(2);
        IllegalArgumentException centerError = assertThrows(IllegalArgumentException.class,
                () -> CoordCubeMap.proximity(map, path(300, 300), 1, 2, 1),
                "proximity rejects out-of-domain center");
        assertTrue(centerError.getMessage().contains("outside the map byte-space domain"),
                centerError.getMessage());

        IllegalArgumentException rangeError = assertThrows(IllegalArgumentException.class,
                () -> CoordCubeMap.boundingBoxRange(map, new int[][] {{0, 255}, {0, 300}}),
                "bounding box rejects out-of-domain range");
        assertTrue(rangeError.getMessage().contains("outside the map byte-space domain"),
                rangeError.getMessage());
    }

    @Test
    void dimensionMismatchRejected() {
        CoordMapN map = new CoordMapN(2);
        assertThrows(IllegalArgumentException.class,
                () -> CoordCubeMap.proximity(map, path(5, 5), 1, 2, 2),
                "cube interpretation must match the path length");
    }

    @Test
    void nonPositiveInterpretationRejected() {
        CoordMapN map = new CoordMapN(2);
        assertThrows(IllegalArgumentException.class,
                () -> CoordCubeMap.proximity(map, CoordPath.fromArray(), 0, -1, 0),
                "negative dimension count rejected");
        assertThrows(IllegalArgumentException.class,
                () -> CoordCubeMap.proximity(map, path(5, 5), 1, 0, 2),
                "zero dimension count rejected");
        assertThrows(IllegalArgumentException.class,
                () -> CoordCubeMap.proximity(map, path(5, 5), 1, 2, 0),
                "zero resolution rejected");
    }

    @Test
    void wrongLengthCenterYieldsNoHits() {
        CoordMap2 map = new CoordMap2();
        map.insert("hi", bytes("v"));

        // A center that cannot address the store is absent by definition, so
        // the query reports no hits instead of an exception.
        assertTrue(CoordCubeMap.proximity(map, path(1, 2, 3), 0, 3, 1).isEmpty(),
                "longer center cannot address the store");
        assertTrue(CoordCubeMap.proximity(map, path(1), 0, 1, 1).isEmpty(),
                "shorter center cannot address the store");
        assertTrue(CoordCubeMap.boundingBoxRange(map, new int[][] {{1, 2}, {1, 2}, {1, 2}}).isEmpty(),
                "longer range set cannot address the store");

        // The store itself is reachable with a matching center.
        assertEquals(1, CoordCubeMap.proximity(map, path('h', 'i'), 0, 2, 1).size(),
                "matching center finds the entry");
    }

    @Test
    void hitEqualityIsValueBased() {
        CoordCubeMap.Hit a = new CoordCubeMap.Hit(path(5, 5), bytes("v"));
        CoordCubeMap.Hit b = new CoordCubeMap.Hit(path(5, 5), bytes("v"));
        CoordCubeMap.Hit c = new CoordCubeMap.Hit(path(5, 5), bytes("w"));
        assertEquals(a, b, "equal hits");
        assertEquals(a.hashCode(), b.hashCode(), "equal hits share a hash");
        assertNotEquals(a, c, "different hit values");
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

    private static List<CoordPath> pathsOf(List<CoordCubeMap.Hit> hits) {
        List<CoordPath> paths = new ArrayList<>(hits.size());
        for (CoordCubeMap.Hit hit : hits) {
            paths.add(hit.path());
        }
        return paths;
    }
}
