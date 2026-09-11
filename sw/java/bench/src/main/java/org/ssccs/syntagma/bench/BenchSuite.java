package org.ssccs.syntagma.bench;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Optional;

import org.ssccs.syntagma.core.Coord;
import org.ssccs.syntagma.core.CoordCube;
import org.ssccs.syntagma.core.CoordPath;
import org.ssccs.syntagma.core.CoordSetN;
import org.ssccs.syntagma.core.CoordSpace;
import org.ssccs.syntagma.core.CoordSpaceM;
import org.ssccs.syntagma.core.CoordSpaceN;
import org.ssccs.syntagma.geo.SpatialOps;
import org.ssccs.syntagma.map.CoordCubeMap;
import org.ssccs.syntagma.map.CoordKey;
import org.ssccs.syntagma.map.CoordMapN;
import org.ssccs.syntagma.map.DynCoordMap;

/**
 * The scenario families of the suite, the {@code bench_*} functions of
 * {@code sw/cpp/bench/bench.cpp}, which Java carries as a static facade because
 * the language has no free functions. The scenario names and the iteration and
 * round counts passed to the harness are the ones of the C++ suite, so a Java
 * result file is comparable scenario by scenario with a C++ one; the scenario
 * set itself follows {@code sw/rust/benches/bench.rs}.
 *
 * <p>Where the C++ suite varies an input per call so the optimizer cannot hoist
 * the query out of the timed loop, the Java suite varies it the same way: the
 * rotating {@code cursor} of the cube, distance and spatial-map scenarios, and
 * the rotating key index of the map lookup scenarios. Every scenario deposits
 * its observable result in {@link BenchHarness#sink()}.
 *
 * <p>Porting differences, all of them deliberate:
 *
 * <ul>
 *   <li>The {@code CoordSpaceM} scenarios keep their paths inside the first
 *       window and close the space with try-with-resources. The space is
 *       anonymous off-heap memory that materializes one window per touched
 *       window, so the reference {@code paths_3d} addresses, whose leading
 *       coordinate advances by {@code 11172^2} slots, would need hundreds of
 *       gigabytes of direct memory; the mmap-backed references reserve the
 *       whole region once and pay only for the pages they touch.
 *       try-with-resources drops the windows when the scenario ends, which is
 *       the C++ destructor equivalent.</li>
 *   <li>The {@code csm insert 1k n3} scenario materializes its slot window
 *       before the timed region, so the recorded number is the cost of the
 *       placements rather than the cost of the window reservation. Its javadoc
 *       states exactly what that region contains.</li>
 *   <li>The {@code csn2 insert all 10k} scenario allocates a whole tree per
 *       round, so its Java figure carries garbage-collection variance: the
 *       standard deviation printed beside it is on the order of the mean, and
 *       the mean is only meaningful with it. Measured once the warmup policy
 *       reaches steady state, the scenario reports about 17.4 ms at five rounds
 *       against about 11.4 ms at twenty, the spread of an allocation-heavy loop
 *       rather than of a cold compiler. The C++ figure has neither source of
 *       variance.</li>
 *   <li>The map scenarios that build keys from raw bytes index those bytes
 *       through {@link CoordKey}, because a Java {@code String} carries
 *       characters and its UTF-8 encoding would turn a byte at or above 0x80
 *       into two bytes, changing the key length and the stored path. The byte
 *       values of the keys are the ones of the C++ suite; only the route to the
 *       store differs, {@code insertByCoordKey} and {@code getByCoordKey}
 *       instead of the string entry points.</li>
 *   <li>The two ASCII scenarios, the {@code "hi"} insert and the
 *       {@code "key<i>"} dynamic lookups, keep the string entry points, which
 *       are the exact C++ calls.</li>
 * </ul>
 */
public final class BenchSuite {

    private BenchSuite() {
    }

    /**
     * Runs every scenario family in the order of the C++ {@code main}:
     * {@code CoordSpaceN}, the dense {@code CoordSpace}, {@code CoordSetN}, the
     * anonymous off-heap {@code CoordSpaceM}, the spatial queries and distance
     * metrics of {@code tagma-geo}, and the {@code tagma-map} insert, get and
     * spatial scenarios.
     *
     * @throws NullPointerException when {@code harness} is null
     */
    public static void runAll(BenchHarness harness) {
        Objects.requireNonNull(harness, "harness");
        csn2InsertAll(harness);
        csn2GetAll(harness);
        csn2OverwriteAll(harness);
        csn2FillRemoveAll(harness);
        csn2Iter(harness);
        csn2Mixed(harness);
        spaceEntryOrInsert(harness);
        spaceFillRetainHalf(harness);
        setnInsert1000(harness);
        setnUnion(harness);
        setnIter(harness);
        csmInsert1000(harness);
        csmGet1000(harness);
        cubeProximity(harness);
        cubeBoundingBox(harness);
        cubeProximityHamming(harness);
        cubeDistances(harness);
        mapSingleInsertStatic(harness);
        mapSingleGetStatic(harness);
        mapSingleInsertDyn(harness);
        mapSingleGetDyn(harness);
        mapBatchInsert2k(harness);
        mapSpatialProximity(harness);
    }

    // ------------------------------------------------------------------
    // core: CoordSpaceN
    // ------------------------------------------------------------------

    private static void csn2InsertAll(BenchHarness harness) {
        List<CoordPath> paths = BenchInputs.paths2d(10000);
        harness.run("csn2 insert all 10k", 1, 5, () -> {
            CoordSpaceN<Integer> space = new CoordSpaceN<>(2);
            for (CoordPath path : paths) {
                space.placePath(path, 1);
            }
            harness.sink().add(space.size());
        });
    }

    private static void csn2GetAll(BenchHarness harness) {
        List<CoordPath> paths = BenchInputs.paths2d(10000);
        CoordSpaceN<Integer> space = new CoordSpaceN<>(2);
        for (CoordPath path : paths) {
            space.placePath(path, 1);
        }
        harness.run("csn2 get all 10k", 3, 3, () -> {
            long sum = 0;
            for (CoordPath path : paths) {
                Optional<Integer> value = space.atPath(path);
                if (value.isPresent()) {
                    sum += value.get();
                }
            }
            harness.sink().add(sum);
        });
    }

    private static void csn2OverwriteAll(BenchHarness harness) {
        List<CoordPath> paths = BenchInputs.paths2d(10000);
        CoordSpaceN<Integer> space = new CoordSpaceN<>(2);
        for (CoordPath path : paths) {
            space.placePath(path, 1);
        }
        harness.run("csn2 overwrite all 10k", 3, 3, () -> {
            for (CoordPath path : paths) {
                space.placePath(path, 2);
            }
            harness.sink().add(space.size());
        });
    }

    private static void csn2FillRemoveAll(BenchHarness harness) {
        List<CoordPath> paths = BenchInputs.paths2d(10000);
        harness.run("csn2 fill+remove all 10k", 1, 5, () -> {
            CoordSpaceN<Integer> space = new CoordSpaceN<>(2);
            for (CoordPath path : paths) {
                space.placePath(path, 1);
            }
            for (CoordPath path : paths) {
                space.vacatePath(path);
            }
            harness.sink().add(space.size());
        });
    }

    private static void csn2Iter(BenchHarness harness) {
        List<CoordPath> paths = BenchInputs.paths2d(10000);
        CoordSpaceN<Integer> space = new CoordSpaceN<>(2);
        for (CoordPath path : paths) {
            space.placePath(path, 1);
        }
        harness.run("csn2 iter 10k entries", 3, 3, () -> {
            List<Map.Entry<CoordPath, Integer>> entries = space.entries();
            harness.sink().add(entries.size());
        });
    }

    private static void csn2Mixed(BenchHarness harness) {
        List<CoordPath> paths = BenchInputs.paths2d(5000);
        harness.run("csn2 mixed 5k", 1, 5, () -> {
            CoordSpaceN<Integer> space = new CoordSpaceN<>(2);
            for (int i = 0; i < 5000; i++) {
                space.placePath(paths.get(i), 1);
            }
            long sum = 0;
            for (int i = 0; i < 5000; i += 2) {
                Optional<Integer> value = space.atPath(paths.get(i));
                if (value.isPresent()) {
                    sum += value.get();
                }
            }
            for (int i = 0; i < 5000; i += 2) {
                space.vacatePath(paths.get(i));
            }
            harness.sink().add(sum + space.size());
        });
    }

    // ------------------------------------------------------------------
    // core: dense CoordSpace (N=1)
    // ------------------------------------------------------------------

    private static void spaceEntryOrInsert(BenchHarness harness) {
        harness.run("space entry or_insert 10k", 1, 5, () -> {
            CoordSpace<Integer> space = new CoordSpace<>();
            for (int i = 0; i < 10000; i++) {
                space.entry(BenchInputs.coord(i % Coord.N_VALID)).orInsert(1);
            }
            harness.sink().add(space.size());
        });
    }

    private static void spaceFillRetainHalf(BenchHarness harness) {
        harness.run("space fill+retain half 10k", 1, 5, () -> {
            CoordSpace<Integer> space = new CoordSpace<>();
            for (int i = 0; i < 10000; i++) {
                space.place(BenchInputs.coord(i % Coord.N_VALID), 1);
            }
            space.retain((coord, value) -> coord.index() % 2 == 0);
            harness.sink().add(space.size());
        });
    }

    // ------------------------------------------------------------------
    // core: CoordSetN
    // ------------------------------------------------------------------

    private static void setnInsert1000(BenchHarness harness) {
        List<CoordPath> paths = BenchInputs.paths2d(1000);
        harness.run("setn insert 1k n2", 1, 5, () -> {
            CoordSetN set = new CoordSetN(2);
            for (CoordPath path : paths) {
                set.insert(path);
            }
            harness.sink().add(set.size());
        });
    }

    private static void setnUnion(BenchHarness harness) {
        List<CoordPath> aPaths = BenchInputs.paths2d(500);
        List<CoordPath> bPaths = BenchInputs.paths2d(1000);
        CoordSetN a = new CoordSetN(2);
        CoordSetN b = new CoordSetN(2);
        for (CoordPath path : aPaths) {
            a.insert(path);
        }
        for (CoordPath path : bPaths) {
            b.insert(path);
        }
        harness.run("setn union 500+1k n2", 3, 3, () -> {
            CoordSetN union = a.union(b);
            harness.sink().add(union.size());
        });
    }

    private static void setnIter(BenchHarness harness) {
        List<CoordPath> paths = BenchInputs.paths2d(1000);
        CoordSetN set = new CoordSetN(2);
        for (CoordPath path : paths) {
            set.insert(path);
        }
        harness.run("setn iter 1k n2", 3, 3, () -> {
            List<CoordPath> pathsOfSet = set.paths();
            harness.sink().add(pathsOfSet.size());
        });
    }

    // ------------------------------------------------------------------
    // core: CoordSpaceM (anonymous off-heap)
    // ------------------------------------------------------------------

    /**
     * The dense-space paths of the csm scenarios. The reference scenarios walk
     * {@link BenchInputs#paths3d}, whose leading coordinate advances by
     * {@code 11172^2} slots, so 1000 paths span about 623 GB of the slot
     * region. The anonymous-mapping references pay only for the pages they
     * write, while the Java port materializes one window per touched window and
     * accounts it against {@code -XX:MaxDirectMemorySize}, so that span would
     * need 291 windows, about 582 GiB. The scenarios therefore keep the
     * 1000-insert and 1000-lookup workload and place the same number of
     * distinct 3D addresses inside the first window, advancing the leading
     * coordinate over a range that fits and the second coordinate for the rest,
     * so the writes stay spread out instead of walking the space sequentially.
     */
    private static List<CoordPath> csmPaths3d(int count) {
        List<CoordPath> out = new ArrayList<>(count);
        for (int i = 0; i < count; i++) {
            out.add(CoordPath.fromArray(BenchInputs.coord(i % 4),
                    BenchInputs.coord((i / 4) % Coord.N_VALID), BenchInputs.coord(0)));
        }
        return out;
    }

    /**
     * The {@code csm insert 1k n3} scenario. The measured region contains the
     * 1000 placements and nothing else: the space and its first slot window are
     * materialized before the harness starts timing, because the Java
     * {@code CoordSpaceM} reserves a window of about two gigabytes of direct
     * memory on first touch and faults its pages, which costs on the order of
     * two hundred milliseconds and would dominate a placement of a few dozen
     * nanoseconds. The C++ suite creates the space inside the timed body, where
     * the {@code MAP_NORESERVE} mapping commits only the pages it writes. The
     * first placement of a round therefore lands on the slot that the
     * materializing placement already filled, and the other 999 are fresh
     * insertions into the materialized window.
     */
    private static void csmInsert1000(BenchHarness harness) {
        List<CoordPath> paths = csmPaths3d(1000);
        try (CoordSpaceM<Integer> space = new CoordSpaceM<>(3, Integer.class)) {
            space.placePath(paths.get(0), 1); // materialize the first window before timing
            harness.run("csm insert 1k n3", 1, 5, () -> {
                for (CoordPath path : paths) {
                    space.placePath(path, 1);
                }
                harness.sink().add(space.size());
            });
        }
    }

    private static void csmGet1000(BenchHarness harness) {
        List<CoordPath> paths = csmPaths3d(1000);
        try (CoordSpaceM<Integer> space = new CoordSpaceM<>(3, Integer.class)) {
            for (CoordPath path : paths) {
                space.placePath(path, 1);
            }
            harness.run("csm get 1k n3", 3, 3, () -> {
                long sum = 0;
                for (CoordPath path : paths) {
                    Optional<Integer> value = space.atPath(path);
                    if (value.isPresent()) {
                        sum += value.get();
                    }
                }
                harness.sink().add(sum);
            });
        }
    }

    // ------------------------------------------------------------------
    // tagma_geo
    // ------------------------------------------------------------------

    private static void cubeProximity(BenchHarness harness) {
        List<CoordPath> centers = BenchInputs.paths2d(50);
        int[] cursor = {0};
        harness.run("cube proximity r3 (49 paths)", 1000, 3, () -> {
            cursor[0] = (cursor[0] + 1) % centers.size();
            CoordCube cube = CoordCube.fromPath(2, 1, centers.get(cursor[0]));
            long count = 0;
            for (CoordPath path : SpatialOps.proximity(cube, 3)) {
                count += 1;
            }
            harness.sink().add(count);
        });
    }

    private static void cubeBoundingBox(BenchHarness harness) {
        CoordCube cube = CoordCube.fromPath(2, 1,
                CoordPath.fromArray(BenchInputs.coord(0), BenchInputs.coord(0)));
        int[][][] rangeSets = new int[10][][];
        for (int i = 0; i < rangeSets.length; i++) {
            int lo = i * 10;
            rangeSets[i] = new int[][] {{lo, lo + 50}, {lo, lo + 50}};
        }
        int[] cursor = {0};
        harness.run("cube bounding box 51x51", 100, 3, () -> {
            cursor[0] = (cursor[0] + 1) % rangeSets.length;
            long count = 0;
            for (CoordPath path : SpatialOps.boundingBox(cube, rangeSets[cursor[0]])) {
                count += 1;
            }
            harness.sink().add(count);
        });
    }

    private static void cubeProximityHamming(BenchHarness harness) {
        List<CoordPath> centers = BenchInputs.paths2d(50);
        int[] cursor = {0};
        harness.run("cube proximity hamming r1", 1000, 3, () -> {
            cursor[0] = (cursor[0] + 1) % centers.size();
            CoordCube cube = CoordCube.fromPath(2, 1, centers.get(cursor[0]));
            long count = 0;
            for (CoordPath path : SpatialOps.proximityHamming(cube, 1)) {
                count += 1;
            }
            harness.sink().add(count);
        });
    }

    private static void cubeDistances(BenchHarness harness) {
        List<CoordPath> aPaths = BenchInputs.paths2d(50);
        List<CoordPath> bPaths = BenchInputs.paths2d(50);
        int[] cursor = {0};
        harness.run("cube distance metrics x100", 100, 3, () -> {
            cursor[0] = (cursor[0] + 1) % aPaths.size();
            long sum = 0;
            for (int i = 0; i < 100; i++) {
                CoordCube a = CoordCube.fromPath(2, 1, aPaths.get((cursor[0] + i) % aPaths.size()));
                CoordCube b = CoordCube.fromPath(2, 1, bPaths.get((cursor[0] + i + 7) % bPaths.size()));
                sum += SpatialOps.hammingDistance(a, b);
                sum += (long) (SpatialOps.euclideanDistanceApprox(a, b) * 1e9);
                sum += SpatialOps.manhattanDistance(a, b);
            }
            harness.sink().add(sum);
        });
    }

    // ------------------------------------------------------------------
    // tagma_map
    // ------------------------------------------------------------------

    private static void mapSingleInsertStatic(BenchHarness harness) {
        harness.run("map static single insert", 100, 3, () -> {
            CoordMapN map = new CoordMapN(2);
            map.insert("hi", new byte[] {1});
            harness.sink().add(map.len());
        });
    }

    private static void mapSingleGetStatic(BenchHarness harness) {
        CoordMapN map = new CoordMapN(2);
        List<CoordKey> keys = new ArrayList<>(100);
        for (int i = 0; i < 100; i++) {
            CoordKey key = CoordKey.fromIndices(new int[] {(i * 3) % 256, (i * 5) % 256});
            keys.add(key);
            map.insertByCoordKey(key, new byte[] {1});
        }
        int[] cursor = {0};
        harness.run("map static single get", 100000, 3, () -> {
            cursor[0] = (cursor[0] + 1) % keys.size();
            Optional<byte[]> value = map.getByCoordKey(keys.get(cursor[0]));
            if (value.isPresent()) {
                harness.sink().add(value.get()[0]);
            }
        });
    }

    private static void mapSingleInsertDyn(BenchHarness harness) {
        harness.run("map dyn single insert", 100, 3, () -> {
            DynCoordMap map = new DynCoordMap();
            map.insert("hello", new byte[] {1});
            harness.sink().add(map.len());
        });
    }

    private static void mapSingleGetDyn(BenchHarness harness) {
        DynCoordMap map = new DynCoordMap();
        List<String> keys = new ArrayList<>(100);
        for (int i = 0; i < 100; i++) {
            String key = "key" + i;
            keys.add(key);
            map.insert(key, new byte[] {1});
        }
        int[] cursor = {0};
        harness.run("map dyn single get", 100000, 3, () -> {
            cursor[0] = (cursor[0] + 1) % keys.size();
            Optional<byte[]> value = map.get(keys.get(cursor[0]));
            if (value.isPresent()) {
                harness.sink().add(value.get()[0]);
            }
        });
    }

    private static void mapBatchInsert2k(BenchHarness harness) {
        List<CoordKey> keys = new ArrayList<>(2048);
        for (int i = 0; i < 2048; i++) {
            keys.add(CoordKey.fromIndices(new int[] {i % 256, (i / 256) % 256}));
        }
        harness.run("map static batch insert 2k", 1, 5, () -> {
            CoordMapN map = new CoordMapN(2);
            for (CoordKey key : keys) {
                map.insertByCoordKey(key, new byte[] {1});
            }
            harness.sink().add(map.len());
        });
    }

    private static void mapSpatialProximity(BenchHarness harness) {
        CoordMapN map = new CoordMapN(2);
        for (int i = 0; i < 1000; i++) {
            map.insertByCoordKey(
                    CoordKey.fromIndices(new int[] {(i * 7) % 256, (i * 13) % 256}), new byte[] {1});
        }
        List<CoordPath> centers = BenchInputs.paths2d(50);
        int[] cursor = {0};
        harness.run("map spatial proximity r2 (1k entries)", 100, 3, () -> {
            cursor[0] = (cursor[0] + 1) % centers.size();
            List<CoordCubeMap.Hit> results =
                    CoordCubeMap.proximity(map, centers.get(cursor[0]), 2, 2, 1);
            harness.sink().add(results.size());
        });
    }
}
