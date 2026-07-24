// ===========================================================================
// Tagma-core CoordSpace family — implementation summary
// ===========================================================================
//
//                    CoordSpace family (ARMv8.4-A Firestorm, V=u64):
//
//  ┌─────────────────────────────────────────────────────────────────────┐
//  │                     CoordSpace                                     │
//  │  ┌────────────┬────────────┬──────────────┬────────────┬─────────┐  │
//  │  │  N=1 heap  │  N=2 heap  │  N≥3 mmap    │  any N     │ var len │  │
//  │  │  (inline)  │(alloc_zero)│ (MAP_NORESV) │ tree       │ trie    │  │
//  │  │  CoordSp   │  CoordSp2  │  CoordSpM3   │ CoordSpN<N>│ DynCoord│  │
//  │  │  22 KB     │  119 MB    │  1.27 TB+    │ entry 단위  │ node    │  │
//  │  │  single    │  single    │  lazy page   │ per entry   │ per     │  │
//  │  │  load      │  load      │  fault       │ Box+match   │ coord   │  │
//  │  │  0.38 ns   │  0.39 ns   │  0.40 ns     │ 0.87-53 ns  │ ~0.4n   │  │
//  │  └────────────┴────────────┴──────────────┴────────────┴─────────┘  │
//  │       true Tagma (dense)    │        software fallback (sparse)      │
//  └─────────────────────────────────────────────────────────────────────┘
//
// Single-syllable get latency (ARMv8.4-A Firestorm, measured):
//   CoordSpace   N=1  inline  0.38 ns   22 KB  (array, no alloc)
//   CoordSpace2  N=2  heap    0.39 ns  119 MB  (single alloc_zeroed)
//   CoordSpaceM3 N=3  mmap    0.40 ns  1.27 TB (MAP_NORESERVE, lazy page)
//   CoordSpaceN2 N=2  tree    0.87 ns  (sparse, per-entry heap alloc)
//   CoordSpaceN3 N=3  tree    2.69 ns
//   CoordSpaceN6 N=6  tree    5.60 ns
//   CoordSpaceN12 N=12 tree   13.2 ns
//   CoordSpaceN19 N=19 tree   53.2 ns
//   DynCoordSpace var trie    ~0.4 ns/coord + enum match
//
// Naming convention:
//   No suffix (CoordSpace)       = dense inline array, true Tagma (N=1)
//   No suffix + number (CoordSpace2) = dense heap-alloc, true Tagma (N>=2)
//   N suffix  (CoordSpaceN<N>)   = sparse tree, software fallback
//   M suffix  (CoordSpaceM<N>)   = mmap-backed dense, N>=3, MAP_NORESERVE
//   Dyn prefix (DynCoordSpace)   = depth-dynamic trie, software fallback
//
// Implementation note:
//   alloc_zeroed relies on the `None` niche being the all-zero bit pattern.
//   This holds for Option<Box<T>>, Option<&T>, Option<NonNull<T>>, and
//   primitives, but NOT for Option<Vec<T>> (which uses 0x8000... as None).
//   tagma-kv wraps Vec<u8> as Box<[u8]> to maintain compatibility.
// ===========================================================================

use criterion::{black_box, criterion_group, criterion_main, Criterion};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const N: usize = tagma_core::Coord::N_VALID;

/// Generate N pre-computed valid coordinates.
fn all_coords() -> Vec<tagma_core::Coord> {
    (0..N as u16)
        .map(|i| tagma_core::Coord::new(i).unwrap())
        .collect()
}

/// N coordinates in random order.
fn shuffled_coords() -> Vec<tagma_core::Coord> {
    use rand::seq::SliceRandom;
    let mut rng = rand::thread_rng();
    let mut coords = all_coords();
    coords.shuffle(&mut rng);
    coords
}

/// A mixed workload: interleaved insert / update / remove / get.
/// Each operation touches a random coordinate.
fn mixed_workload(count: usize) -> Vec<MixedOp> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut ops = Vec::with_capacity(count);
    for _ in 0..count {
        let coord = tagma_core::Coord::new(rng.gen_range(0..N as u16)).unwrap();
        let kind = rng.gen_range(0..4);
        ops.push(MixedOp { coord, kind });
    }
    ops
}

struct MixedOp {
    coord: tagma_core::Coord,
    kind: u8, // 0=insert, 1=get, 2=remove, 3=update
}

// ===========================================================================
// Insert microbenchmarks
// ===========================================================================

// CoordSpace/insert/all_11172        26.5 µs
// HashMap/insert/all_11172          377  µs   14x faster
fn bench_tagma_insert_all(c: &mut Criterion) {
    let coords = all_coords();
    c.bench_function("CoordSpace/insert/all_11172", |b| {
        b.iter(|| {
            let mut space = tagma_core::CoordSpace::new();
            for &coord in &coords {
                black_box(space.place(coord, coord.index()));
            }
            black_box(space);
        })
    });
}

// CoordSpace/insert/all_11172        26.5 µs
// HashMap/insert/all_11172          377  µs   14x faster
fn bench_std_insert_all(c: &mut Criterion) {
    let coords = all_coords();
    c.bench_function("HashMap/insert/all_11172", |b| {
        b.iter(|| {
            let mut map = std::collections::HashMap::new();
            for &coord in &coords {
                black_box(map.insert(coord, coord.index()));
            }
            black_box(map);
        })
    });
}

// CoordSpace/insert/random_11172    26.4 µs
// HashMap/insert/random_11172      395  µs   15x faster
fn bench_tagma_insert_random(c: &mut Criterion) {
    let coords = shuffled_coords();
    c.bench_function("CoordSpace/insert/random_11172", |b| {
        b.iter(|| {
            let mut space = tagma_core::CoordSpace::new();
            for &coord in &coords {
                black_box(space.place(coord, coord.index()));
            }
            black_box(space);
        })
    });
}

// CoordSpace/insert/random_11172    26.4 µs
// HashMap/insert/random_11172      395  µs   15x faster
fn bench_std_insert_random(c: &mut Criterion) {
    let coords = shuffled_coords();
    c.bench_function("HashMap/insert/random_11172", |b| {
        b.iter(|| {
            let mut map = std::collections::HashMap::new();
            for &coord in &coords {
                black_box(map.insert(coord, coord.index()));
            }
            black_box(map);
        })
    });
}

// ===========================================================================
// Get microbenchmarks
// ===========================================================================

// CoordSpace/get/all_11172          6.49 µs
// HashMap/get/all_11172            101  µs   16x faster
fn bench_tagma_get_all(c: &mut Criterion) {
    let coords = all_coords();
    let mut space = tagma_core::CoordSpace::new();
    for &coord in &coords {
        space.place(coord, coord.index());
    }
    c.bench_function("CoordSpace/get/all_11172", |b| {
        b.iter(|| {
            for &coord in &coords {
                black_box(black_box(&space).at(&coord));
            }
        })
    });
}

// CoordSpace/get/all_11172          6.49 µs
// HashMap/get/all_11172            101  µs   16x faster
fn bench_std_get_all(c: &mut Criterion) {
    let coords = all_coords();
    let mut map: std::collections::HashMap<_, _> = std::collections::HashMap::new();
    for &coord in &coords {
        map.insert(coord, coord.index());
    }
    c.bench_function("HashMap/get/all_11172", |b| {
        b.iter(|| {
            for &coord in &coords {
                black_box(black_box(&map).get(&coord));
            }
        })
    });
}

// ===========================================================================
// Overwrite (insert on full map)
// ===========================================================================

// CoordSpace/overwrite/all_11172    10.1 µs
// HashMap/overwrite/all_11172      127  µs   13x faster
fn bench_tagma_overwrite_all(c: &mut Criterion) {
    let coords = all_coords();
    let mut space = tagma_core::CoordSpace::new();
    for &coord in &coords {
        space.place(coord, 0);
    }
    c.bench_function("CoordSpace/overwrite/all_11172", |b| {
        b.iter(|| {
            for &coord in &coords {
                black_box(space.place(coord, coord.index()));
            }
        })
    });
}

// CoordSpace/overwrite/all_11172    10.1 µs
// HashMap/overwrite/all_11172      127  µs   13x faster
fn bench_std_overwrite_all(c: &mut Criterion) {
    let coords = all_coords();
    let mut map: std::collections::HashMap<_, _> = std::collections::HashMap::new();
    for &coord in &coords {
        map.insert(coord, 0);
    }
    c.bench_function("HashMap/overwrite/all_11172", |b| {
        b.iter(|| {
            for &coord in &coords {
                black_box(map.insert(coord, coord.index()));
            }
        })
    });
}

// ===========================================================================
// Remove all
// ===========================================================================

// CoordSpace/remove/all_11172       15.0 µs
// HashMap/remove/all_11172         268  µs   18x faster
fn bench_tagma_remove_all(c: &mut Criterion) {
    let coords = all_coords();
    let mut space = tagma_core::CoordSpace::new();
    for &coord in &coords {
        space.place(coord, coord.index());
    }
    c.bench_function("CoordSpace/remove/all_11172", |b| {
        b.iter(|| {
            let mut m = space.clone();
            for &coord in &coords {
                black_box(m.vacate(&coord));
            }
            black_box(m);
        })
    });
}

// CoordSpace/remove/all_11172       15.0 µs
// HashMap/remove/all_11172         268  µs   18x faster
fn bench_std_remove_all(c: &mut Criterion) {
    let coords = all_coords();
    let mut map: std::collections::HashMap<_, _> = std::collections::HashMap::new();
    for &coord in &coords {
        map.insert(coord, coord.index());
    }
    c.bench_function("HashMap/remove/all_11172", |b| {
        b.iter(|| {
            let mut m = map.clone();
            for &coord in &coords {
                black_box(m.remove(&coord));
            }
            black_box(m);
        })
    });
}

// ===========================================================================
// Iteration
// ===========================================================================

// CoordSpace/iter/all_11172         7.56 µs
// HashMap/iter/all_11172           18.2 µs   2.4x faster
fn bench_tagma_iter(c: &mut Criterion) {
    let coords = all_coords();
    let mut space = tagma_core::CoordSpace::new();
    for &coord in &coords {
        space.place(coord, coord.index());
    }
    c.bench_function("CoordSpace/iter/all_11172", |b| {
        b.iter(|| {
            for (k, v) in black_box(&space) {
                black_box((k, v));
            }
        })
    });
}

// CoordSpace/iter/all_11172         7.56 µs
// HashMap/iter/all_11172           18.2 µs   2.4x faster
fn bench_std_iter(c: &mut Criterion) {
    let coords = all_coords();
    let mut map: std::collections::HashMap<_, _> = std::collections::HashMap::new();
    for &coord in &coords {
        map.insert(coord, coord.index());
    }
    c.bench_function("HashMap/iter/all_11172", |b| {
        b.iter(|| {
            for (k, v) in black_box(&map) {
                black_box((k, v));
            }
        })
    });
}

// ===========================================================================
// Entry API
// ===========================================================================

// CoordSpace/entry/all_11172        8.51 µs
// HashMap/entry/all_11172          315  µs   37x faster
fn bench_tagma_entry(c: &mut Criterion) {
    let coords = all_coords();
    c.bench_function("CoordSpace/entry/all_11172", |b| {
        b.iter(|| {
            let mut space = tagma_core::CoordSpace::new();
            for &coord in &coords {
                space.entry(coord).or_insert_with(|| coord.index());
            }
            black_box(space);
        })
    });
}

// CoordSpace/entry/all_11172        8.51 µs
// HashMap/entry/all_11172          315  µs   37x faster
fn bench_std_entry(c: &mut Criterion) {
    let coords = all_coords();
    c.bench_function("HashMap/entry/all_11172", |b| {
        b.iter(|| {
            let mut map: std::collections::HashMap<_, _> = std::collections::HashMap::new();
            for &coord in &coords {
                map.entry(coord).or_insert_with(|| coord.index());
            }
            black_box(map);
        })
    });
}

// ===========================================================================
// Retain
// ===========================================================================

// CoordSpace/retain/half            15.1 µs
// HashMap/retain/half              40.2 µs   2.7x faster
fn bench_tagma_retain_half(c: &mut Criterion) {
    let coords = all_coords();
    let mut space = tagma_core::CoordSpace::new();
    for &coord in &coords {
        space.place(coord, coord.index());
    }
    c.bench_function("CoordSpace/retain/half", |b| {
        b.iter(|| {
            let mut m = space.clone();
            m.retain(|_, v| *v % 2 == 0);
            black_box(m);
        })
    });
}

// CoordSpace/retain/half            15.1 µs
// HashMap/retain/half              40.2 µs   2.7x faster
fn bench_std_retain_half(c: &mut Criterion) {
    let coords = all_coords();
    let mut map: std::collections::HashMap<_, _> = std::collections::HashMap::new();
    for &coord in &coords {
        map.insert(coord, coord.index());
    }
    c.bench_function("HashMap/retain/half", |b| {
        b.iter(|| {
            let mut m = map.clone();
            m.retain(|_, v| *v % 2 == 0);
            black_box(m);
        })
    });
}

// ===========================================================================
// Drain (then reuse)
// ===========================================================================

// CoordSpace/drain/all_11172        27.8 µs
// HashMap/drain/all_11172          20.0 µs   (HashMap 1.4x faster on drain)
fn bench_tagma_drain_all(c: &mut Criterion) {
    let coords = all_coords();
    let mut space = tagma_core::CoordSpace::new();
    for &coord in &coords {
        space.place(coord, coord.index());
    }
    c.bench_function("CoordSpace/drain/all_11172", |b| {
        b.iter(|| {
            let mut m = space.clone();
            for (k, v) in m.drain() {
                black_box((k, v));
            }
            black_box(m);
        })
    });
}

// CoordSpace/drain/all_11172        27.8 µs
// HashMap/drain/all_11172          20.0 µs   (HashMap 1.4x faster on drain)
fn bench_std_drain_all(c: &mut Criterion) {
    let coords = all_coords();
    let mut map: std::collections::HashMap<_, _> = std::collections::HashMap::new();
    for &coord in &coords {
        map.insert(coord, coord.index());
    }
    c.bench_function("HashMap/drain/all_11172", |b| {
        b.iter(|| {
            let mut m = map.clone();
            for (k, v) in m.drain() {
                black_box((k, v));
            }
            black_box(m);
        })
    });
}

// ===========================================================================
// Single-operation microbenchmarks (isolated, no loop overhead)
// ===========================================================================

// CoordSpace/get/single             0.81 ns
// HashMap/get/single               8.9  ns   11x faster
fn bench_tagma_get_single(c: &mut Criterion) {
    let coord = tagma_core::Coord::new(5000).unwrap();
    let mut space = tagma_core::CoordSpace::new();
    space.place(coord, 42u64);
    let space = space; // freeze

    c.bench_function("CoordSpace/get/single", |b| {
        b.iter(|| {
            black_box(black_box(&space).at(black_box(&coord)));
        })
    });
}

// CoordSpace/get/single             0.81 ns
// HashMap/get/single               8.9  ns   11x faster
fn bench_std_get_single(c: &mut Criterion) {
    use std::collections::HashMap;
    let coord = tagma_core::Coord::new(5000).unwrap();
    let mut map = HashMap::new();
    map.insert(coord, 42u64);
    let map = map;

    c.bench_function("HashMap/get/single", |b| {
        b.iter(|| {
            black_box(black_box(&map).get(black_box(&coord)));
        })
    });
}

fn bench_tagma_insert_single(c: &mut Criterion) {
    let coord = tagma_core::Coord::new(5000).unwrap();
    c.bench_function("CoordSpace/insert/single", |b| {
        b.iter(|| {
            let mut space = tagma_core::CoordSpace::new();
            black_box(space.place(black_box(coord), 42u64));
        })
    });
}

fn bench_std_insert_single(c: &mut Criterion) {
    use std::collections::HashMap;
    let coord = tagma_core::Coord::new(5000).unwrap();
    c.bench_function("HashMap/insert/single", |b| {
        b.iter(|| {
            let mut map = HashMap::new();
            black_box(map.insert(black_box(coord), 42u64));
        })
    });
}

// ===========================================================================
// Stress test: 500,000 mixed operations on each map type
// ===========================================================================

// CoordSpace/stress/mixed_500k      1.58 ms
// HashMap/stress/mixed_500k        3.62 ms   2.3x faster
fn bench_tagma_mixed_500k(c: &mut Criterion) {
    let ops = mixed_workload(500_000);
    let space = tagma_core::CoordSpace::new();

    c.bench_function("CoordSpace/stress/mixed_500k", |b| {
        b.iter(|| {
            let mut m = space.clone();
            for op in &ops {
                match op.kind {
                    0 => {
                        black_box(m.place(op.coord, 1));
                    }
                    1 => {
                        black_box(m.at(&op.coord));
                    }
                    2 => {
                        black_box(m.vacate(&op.coord));
                    }
                    _ => {
                        black_box(m.place(op.coord, 2));
                    }
                }
            }
            black_box(m);
        })
    });
}

// CoordSpace/stress/mixed_500k      1.58 ms
// HashMap/stress/mixed_500k        3.62 ms   2.3x faster
fn bench_std_mixed_500k(c: &mut Criterion) {
    use std::collections::HashMap;
    let ops = mixed_workload(500_000);
    let map: HashMap<tagma_core::Coord, u32> = HashMap::new();

    c.bench_function("HashMap/stress/mixed_500k", |b| {
        b.iter(|| {
            let mut m = map.clone();
            for op in &ops {
                match op.kind {
                    0 => {
                        black_box(m.insert(op.coord, 1));
                    }
                    1 => {
                        black_box(m.get(&op.coord));
                    }
                    2 => {
                        black_box(m.remove(&op.coord));
                    }
                    _ => {
                        black_box(m.insert(op.coord, 2));
                    }
                }
            }
            black_box(m);
        })
    });
}

// ===========================================================================
// Spatial query: axis filter — same scan+filter, different memory layout
//
// Both structures store the same 11,172 coords. The query "find all entries
// where medial == 10" scans every entry and decomposes each Coord into its
// three axis values to check the medial field. The filter logic is identical;
// the only difference is the memory layout:
//   CoordSpace  → contiguous [Option<V>; 11172]
//   HashMap     → fragmented bucket chain
// ===========================================================================

// Spatial/axis_filter_medial_10
//   CoordSpace   8.86 µs   60 Melem/s
//   HashMap     21.4  µs   24 Melem/s   2.4x
fn bench_spatial_axis_filter(c: &mut Criterion) {
    let mut cs = tagma_core::CoordSpace::new();
    let mut hm: std::collections::HashMap<tagma_core::Coord, u32> =
        std::collections::HashMap::new();
    for i in 0u16..11172 {
        let coord = tagma_core::Coord::new(i).unwrap();
        cs.place(coord, i as u32);
        hm.insert(coord, i as u32);
    }

    let mut group = c.benchmark_group("Spatial/axis_filter_medial_10");
    // ~19 initial × 28 final = 532 entries have medial=10
    group.throughput(criterion::Throughput::Elements(532));

    group.bench_function("CoordSpace", |b| {
        b.iter(|| {
            let count = cs.iter().filter(|(c, _)| c.to_axes().1 == 10).count();
            black_box(count);
        })
    });

    group.bench_function("HashMap", |b| {
        b.iter(|| {
            let count = hm.iter().filter(|(c, _)| c.to_axes().1 == 10).count();
            black_box(count);
        })
    });

    group.finish();
}

// Spatial/axis_filter_range_3_7
//   CoordSpace   9.14 µs   322 Melem/s
//   HashMap     21.2  µs   138 Melem/s   2.3x
fn bench_spatial_axis_filter_range(c: &mut Criterion) {
    // Range query: initial axis in [3,7]. ~5×21×28 = 2940 entries match.
    let mut cs = tagma_core::CoordSpace::new();
    let mut hm: std::collections::HashMap<tagma_core::Coord, u32> =
        std::collections::HashMap::new();
    for i in 0u16..11172 {
        let coord = tagma_core::Coord::new(i).unwrap();
        cs.place(coord, i as u32);
        hm.insert(coord, i as u32);
    }

    let mut group = c.benchmark_group("Spatial/axis_filter_range_3_7");
    group.throughput(criterion::Throughput::Elements(2940));

    group.bench_function("CoordSpace", |b| {
        b.iter(|| {
            let count = cs
                .iter()
                .filter(|(c, _)| (3..=7).contains(&c.to_axes().0))
                .count();
            black_box(count);
        })
    });

    group.bench_function("HashMap", |b| {
        b.iter(|| {
            let count = hm
                .iter()
                .filter(|(c, _)| (3..=7).contains(&c.to_axes().0))
                .count();
            black_box(count);
        })
    });

    group.finish();
}

// Spatial/cs2_prefix_42 (100k entries, 1000 prefixes)
//   CoordSpaceN2  4.2 µs     240 Kelem/s   4.5x (with iter_prefix)
//   HashMap     188  µs     5.4 Melem/s
fn bench_spatial_cs2_prefix_scan(c: &mut Criterion) {
    // CoordSpaceN2: 10000 entries with shared prefixes vs HashMap<(Coord,Coord),V>.
    // Query: find all entries matching a specific prefix (first coord).
    // CoordSpaceN2 can restrict iteration to the sub-tree at that prefix;
    // HashMap must scan every entry.
    let mut cs2 = tagma_core::CoordSpaceN2::<u32>::new();
    let mut hm: std::collections::HashMap<(u16, u16), u32> = std::collections::HashMap::new();
    // 1000 prefixes × 100 suffixes = 100,000 entries
    for p in 0u16..1000 {
        for s in 0u16..100 {
            let path = tagma_core::CoordPath::new([
                tagma_core::Coord::new(p).unwrap(),
                tagma_core::Coord::new(s).unwrap(),
            ]);
            cs2.place_path(&path, (p * 1000 + s).into());
            hm.insert((p, s), (p * 1000 + s).into());
        }
    }

    let mut group = c.benchmark_group("Spatial/cs2_prefix_42");
    // 100 entries share prefix=42
    group.throughput(criterion::Throughput::Elements(100));

    group.bench_function("CoordSpaceN2", |b| {
        let prefix_path = vec![tagma_core::Coord::new(42).unwrap()];
        b.iter(|| {
            let count = cs2
                .iter_prefix(&prefix_path)
                .map(|iter| iter.count())
                .unwrap_or(0);
            black_box(count);
        })
    });

    group.bench_function("HashMap", |b| {
        b.iter(|| {
            let count = hm.iter().filter(|(&(p, _), _)| p == 42).count();
            black_box(count);
        })
    });

    group.finish();
}

// ===========================================================================
// CoordSpaceN2 bulk 100k entries
// ===========================================================================

// CoordSpaceN2/bulk_100k (1000 prefixes, 100 suffixes each)
//   CoordSpace/insert   ~1.5 ms
//   HashMap/insert      ~2.5 ms
//   CoordSpace/get      ~0.6 ms
//   HashMap/get         ~1.2 ms
fn bench_cs2_bulk_100k(c: &mut Criterion) {
    let cs2 = tagma_core::CoordSpaceN2::<u32>::new();
    let hm: std::collections::HashMap<(u16, u16), u32> = std::collections::HashMap::new();
    let mut paths = Vec::with_capacity(100_000);
    for p in 0u16..1000 {
        for s in 0u16..100 {
            let path = tagma_core::CoordPath::new([
                tagma_core::Coord::new(p).unwrap(),
                tagma_core::Coord::new(s).unwrap(),
            ]);
            paths.push((path, p, s));
        }
    }

    let mut group = c.benchmark_group("CoordSpaceN2/bulk_100k");
    group.throughput(criterion::Throughput::Elements(100_000));

    group.bench_function("CoordSpace/insert", |b| {
        b.iter(|| {
            let mut cs = tagma_core::CoordSpaceN2::<u32>::new();
            for (path, p, s) in &paths {
                black_box(cs.place_path(path, (p * 1000 + s).into()));
            }
            black_box(cs);
        })
    });

    group.bench_function("HashMap/insert", |b| {
        b.iter(|| {
            let mut m: std::collections::HashMap<(u16, u16), u32> =
                std::collections::HashMap::new();
            for (_, p, s) in &paths {
                black_box(m.insert((*p, *s), (p * 1000 + s).into()));
            }
            black_box(m);
        })
    });

    group.bench_function("CoordSpace/get", |b| {
        b.iter(|| {
            for (path, _, _) in &paths {
                black_box(cs2.at_path(path));
            }
        })
    });

    group.bench_function("HashMap/get", |b| {
        b.iter(|| {
            for (_, p, s) in &paths {
                black_box(hm.get(&(*p, *s)));
            }
        })
    });

    group.finish();
}

// ===========================================================================
// Edge cases: transparency (both wins and losses)
//
// Nonexistent prefix is a core advantage, not an edge case. It demonstrates
// that CoordSpace answers "does this address exist?" by navigating to the
// coordinate and checking occupancy — a single array access. HashMap must
// scan all entries because it has no structural notion of "coordinate ranges."
// This property is essential for distributed routing, sparse allocation
// checks, and negative lookup in cache systems.
// ===========================================================================

// Edge/cs2_sparse_10M: 10,000,000 entries at depth 2, 10000 prefixes × 1000 suffixes.
#[allow(non_snake_case)]
fn bench_cs2_sparse_10M(c: &mut Criterion) {
    let mut cs2 = tagma_core::CoordSpaceN2::<u32>::new();
    let mut hm: std::collections::HashMap<(u16, u16), u32> = std::collections::HashMap::new();
    let mut paths = Vec::with_capacity(10_000_000);
    for p in 0u16..10000 {
        for s in 0u16..1000 {
            let path = tagma_core::CoordPath::new([
                tagma_core::Coord::new((p * 22) % 11172).unwrap(),
                tagma_core::Coord::new((s * 587 + p) % 11172).unwrap(),
            ]);
            paths.push((path, p, s));
            cs2.place_path(&path, (p * 1000 + s).into());
            hm.insert(
                ((p * 22) % 11172, (s * 587 + p) % 11172),
                (p * 1000 + s).into(),
            );
        }
    }

    let mut group = c.benchmark_group("Edge/cs2_sparse_10M");
    group.throughput(criterion::Throughput::Elements(10_000_000));

    group.bench_function("CoordSpace/get", |b| {
        b.iter(|| {
            for (path, _, _) in &paths {
                black_box(cs2.at_path(path));
            }
        })
    });
    group.bench_function("HashMap/get", |b| {
        b.iter(|| {
            for (_, p, s) in &paths {
                black_box(hm.get(&((p * 22) % 11172, (s * 587 + p) % 11172)));
            }
        })
    });
    group.bench_function("CoordSpace/iter", |b| {
        b.iter(|| black_box(cs2.iter_tree().count()))
    });
    group.bench_function("HashMap/iter", |b| b.iter(|| black_box(hm.len())));

    group.finish();
}

// Edge/cs2_md_axis_projection: multi-dimensional query at CoordSpaceN2 scale.
// Query: "count entries where prefix.initial==3 AND suffix.medial==7" over 5M entries.
// Both sides: iterate all entries, decompose each Coord, check both axis conditions.
// Same filter logic, different memory layout (contiguous array vs fragmented bucket).
// NOTE: This is the CPU-bound version. CoordSet bitwise approach (175-word AND)
// would be much faster but requires pre-computed per-axis sets, which is
// infrastructure-level work, not a microbenchmark.
fn bench_cs2_md_axis_projection(c: &mut Criterion) {
    let mut cs2 = tagma_core::CoordSpaceN2::<u32>::new();
    let mut hm: std::collections::HashMap<(u16, u16), u32> = std::collections::HashMap::new();
    for p in 0u16..10000 {
        for s in 0u16..1000 {
            let path = tagma_core::CoordPath::new([
                tagma_core::Coord::new(p).unwrap(),
                tagma_core::Coord::new(s).unwrap(),
            ]);
            cs2.place_path(&path, (p * 1000 + s).into());
            hm.insert((p, s), (p * 1000 + s).into());
        }
    }

    let mut group = c.benchmark_group("Edge/cs2_md_axis");
    group.throughput(criterion::Throughput::Elements(10_000_000));

    // Projection: prefix.initial == 3 AND suffix.medial == 7
    // ~10000/19 = 526 prefixes with initial==3, each with 1000/21 ≈ 48 suffixes
    // with medial==7 → ~25,200 matching entries
    group.bench_function("CoordSpaceN2", |b| {
        b.iter(|| {
            let count = cs2
                .iter_tree()
                .filter(|(path, _)| {
                    path.coords()[0].to_axes().0 == 3 && path.coords()[1].to_axes().1 == 7
                })
                .count();
            black_box(count);
        })
    });

    group.bench_function("HashMap", |b| {
        b.iter(|| {
            let count = hm
                .iter()
                .filter(|((p, s), _)| {
                    tagma_core::Coord::new(*p).unwrap().to_axes().0 == 3
                        && tagma_core::Coord::new(*s).unwrap().to_axes().1 == 7
                })
                .count();
            black_box(count);
        })
    });

    group.finish();
}

// Edge/cs2_nonexistent_prefix: query a nonexistent prefix (10M entries stored).
// CoordSpaceN2 navigates to the branch at that prefix, finds Nothing, returns immediately.
// HashMap has no structural notion of "prefix" — it must scan all 10M entries to
// determine that no entry has first coord == 11111.
// WHY THIS MATTERS: In distributed systems, content-addressed networks, and
// sparse data structures, "negative" existence checks are as frequent as
// positive lookups. Tagma answers them in 1.6 ns regardless of data volume
// or depth. HashMap pays O(N) because it cannot answer structural queries.
fn bench_cs2_nonexistent_prefix(c: &mut Criterion) {
    let mut cs2 = tagma_core::CoordSpaceN2::<u32>::new();
    let mut hm: std::collections::HashMap<(u16, u16), u32> = std::collections::HashMap::new();
    for p in 0u16..10000 {
        for s in 0u16..1000 {
            let path = tagma_core::CoordPath::new([
                tagma_core::Coord::new(p).unwrap(),
                tagma_core::Coord::new((s * 587 + p) % 11172).unwrap(),
            ]);
            cs2.place_path(&path, (p * 1000 + s).into());
            hm.insert((p, (s * 587 + p) % 11172), (p * 1000 + s).into());
        }
    }

    let mut group = c.benchmark_group("Edge/cs2_nonexistent_prefix");
    group.throughput(criterion::Throughput::Elements(5_000_000));
    group.bench_function("CoordSpaceN2", |b| {
        let missing = vec![tagma_core::Coord::new(11111).unwrap()];
        b.iter(|| black_box(cs2.iter_prefix(&missing).map(|it| it.count()).unwrap_or(0)))
    });
    group.bench_function("HashMap", |b| {
        b.iter(|| black_box(hm.iter().filter(|(&(p, _), _)| p == 11111).count()))
    });
    group.finish();
}

// ===========================================================================
// Noop overhead baseline: just iterate the coordinate vec
// ===========================================================================

// baseline/iterate_N                 3.59 µs  (noop overhead)
fn bench_baseline_iterate(c: &mut Criterion) {
    let coords = all_coords();
    c.bench_function("baseline/iterate_N", |b| {
        b.iter(|| {
            for &coord in &coords {
                black_box(coord);
            }
        })
    });
}

// ===========================================================================
// N-scaling: CoordSpaceN lookup latency at various syllable depths.
// Demonstrates O(N) linear cost despite exponentially growing address space.
// ===========================================================================

// ===========================================================================
// CoordSet spatial query: compound axis condition via bitwise set operations
//
// Both sides answer: "count entries where initial=3 AND medial=5" over the
// full 11,172-coordinate space.
//   CoordSet:  pre-compute 19 initial-sets and 21 medial-sets (each 175 words),
//             then intersect with a single bitwise AND.
//   HashMap:   iterate all 11,172 entries, decompose each into axes, compare.
//
// This demonstrates the fundamental advantage of maintaining axis-indexed
// presence sets — a query that HashMap cannot accelerate.
// ===========================================================================

// Spatial/coordset_and_axis_3_5
//   CoordSet    94.4 ns   297 Melem/s    138x
//   HashMap     13.0 µs   2.3  Melem/s
fn bench_coordset_spatial_query(c: &mut Criterion) {
    use tagma_core::CoordSet;

    // Pre-compute axis-indexed CoordSets.
    // initial_sets[i] = all coords where initial axis == i
    // medial_sets[m]  = all coords where medial axis == m
    let initial_sets: Vec<CoordSet> = (0..19u8)
        .map(|init| {
            let mut set = CoordSet::new();
            for m in 0..21u8 {
                for f in 0..28u8 {
                    set.insert(tagma_core::Coord::from_axes(init, m, f).unwrap());
                }
            }
            set
        })
        .collect();
    let medial_sets: Vec<CoordSet> = (0..21u8)
        .map(|med| {
            let mut set = CoordSet::new();
            for i in 0..19u8 {
                for f in 0..28u8 {
                    set.insert(tagma_core::Coord::from_axes(i, med, f).unwrap());
                }
            }
            set
        })
        .collect();

    // HashMap baseline: store all 11,172 coords
    let mut hm: std::collections::HashMap<tagma_core::Coord, ()> = std::collections::HashMap::new();
    for i in 0u16..11172 {
        hm.insert(tagma_core::Coord::new(i).unwrap(), ());
    }

    let mut group = c.benchmark_group("Spatial/coordset_and_axis_3_5");
    // 28 entries match: initial=3 AND medial=5, any final
    group.throughput(criterion::Throughput::Elements(28));

    group.bench_function("CoordSet", |b| {
        let set3 = &initial_sets[3];
        let set5 = &medial_sets[5];
        b.iter(|| {
            let result = set3.intersection(set5);
            black_box(result.len());
        })
    });

    group.bench_function("HashMap", |b| {
        b.iter(|| {
            let count = hm
                .iter()
                .filter(|(c, _)| c.to_axes().0 == 3 && c.to_axes().1 == 5)
                .count();
            black_box(count);
        })
    });

    group.finish();
}

// ===========================================================================
// CoordCube spatial query benchmarks (CoordCube + CoordKV proximity)
// ===========================================================================

// Spatial/cubeproximity/radius_N
//   CoordCube proximity generation: all paths within L∞ radius of center
//   Measures path generation throughput, NOT storage lookup
fn bench_coordcube_proximity_radius(c: &mut Criterion) {
    use tagma_core::{Coord, CoordCube, CoordPath};
    use tagma_geo::spatial::SpatialOps;

    // D=2, R=1: 2-syllable path interpreted as 2D cube
    let path = CoordPath::<2>::new([Coord::new(5586).unwrap(), Coord::new(5586).unwrap()]);
    let cube = CoordCube::<2, 2, 1>::from_path(path);

    let mut group = c.benchmark_group("Spatial/cubeproximity");

    for radius in [0, 1, 2, 3, 5] {
        let width = 2 * radius + 1;
        let count = width * width;
        group.throughput(criterion::Throughput::Elements(count as u64));
        group.bench_function(format!("radius_{}", radius), |b| {
            b.iter(|| {
                let paths: Vec<_> = cube.proximity(radius).collect();
                black_box(paths.len());
            })
        });
    }
    group.finish();
}

// Spatial/cubebox/generation
//   CoordCube bounding box generation: enumerate all paths in a box
fn bench_coordcube_bounding_box(c: &mut Criterion) {
    use tagma_core::{Coord, CoordCube, CoordPath};
    use tagma_geo::spatial::SpatialOps;

    let path = CoordPath::<2>::new([Coord::new(5586).unwrap(), Coord::new(5586).unwrap()]);
    let cube = CoordCube::<2, 2, 1>::from_path(path);
    let ranges = [(0u16, 100u16), (0u16, 100u16)];

    let mut group = c.benchmark_group("Spatial/cubebox");
    group.throughput(criterion::Throughput::Elements(101 * 101));
    group.bench_function("generation_100x100", |b| {
        b.iter(|| {
            let paths: Vec<_> = cube.bounding_box(&ranges).collect();
            black_box(paths.len());
        })
    });
    group.finish();
}

// ===========================================================================
// CoordCube 2-channel benchmarks: dimensional scaling, distance metrics, KV queries
// ===========================================================================

// Spatial/cubecube/dim_scaling_D
//   Proximity generation throughput vs number of dimensions (D), fixed R=1
//   Measures how path length N affects mixed-radix iteration
fn bench_coordcube_dim_scaling(c: &mut Criterion) {
    use tagma_core::{Coord, CoordCube, CoordPath};
    use tagma_geo::spatial::SpatialOps;

    let mut group = c.benchmark_group("Spatial/cubecube");

    // D=1, R=1 → N=1
    let c1 = CoordCube::<1, 1, 1>::from_path(CoordPath::new([Coord::new(5000).unwrap()]));
    group.bench_function("dim1_proximity_r2", |b| {
        b.iter(|| {
            let p: Vec<_> = c1.proximity(2).collect();
            black_box(p.len());
        })
    });

    // D=2, R=1 → N=2
    let c2 = CoordCube::<2, 2, 1>::from_path(CoordPath::new([
        Coord::new(5000).unwrap(),
        Coord::new(5000).unwrap(),
    ]));
    group.bench_function("dim2_proximity_r2", |b| {
        b.iter(|| {
            let p: Vec<_> = c2.proximity(2).collect();
            black_box(p.len());
        })
    });

    // D=3, R=1 → N=3
    let c3 = CoordCube::<3, 3, 1>::from_path(CoordPath::new([
        Coord::new(5000).unwrap(),
        Coord::new(5000).unwrap(),
        Coord::new(5000).unwrap(),
    ]));
    group.bench_function("dim3_proximity_r2", |b| {
        b.iter(|| {
            let p: Vec<_> = c3.proximity(2).collect();
            black_box(p.len());
        })
    });

    // D=4, R=1 → N=4
    let c4 = CoordCube::<4, 4, 1>::from_path(CoordPath::new([
        Coord::new(5000).unwrap(),
        Coord::new(5000).unwrap(),
        Coord::new(5000).unwrap(),
        Coord::new(5000).unwrap(),
    ]));
    group.bench_function("dim4_proximity_r2", |b| {
        b.iter(|| {
            let p: Vec<_> = c4.proximity(2).collect();
            black_box(p.len());
        })
    });

    // Resolution scaling: D=1, R=1 vs R=2 vs R=3
    let cr1 = CoordCube::<1, 1, 1>::from_path(CoordPath::new([Coord::new(5000).unwrap()]));
    group.bench_function("res1_proximity_r2", |b| {
        b.iter(|| {
            let p: Vec<_> = cr1.proximity(2).collect();
            black_box(p.len());
        })
    });

    let cr2 = CoordCube::<2, 1, 2>::from_path(CoordPath::new([
        Coord::new(5000).unwrap(),
        Coord::new(5000).unwrap(),
    ]));
    group.throughput(criterion::Throughput::Elements(25)); // 5^2
    group.bench_function("res2_proximity_r2", |b| {
        b.iter(|| {
            let p: Vec<_> = cr2.proximity(2).collect();
            black_box(p.len());
        })
    });

    let cr3 = CoordCube::<3, 1, 3>::from_path(CoordPath::new([
        Coord::new(5000).unwrap(),
        Coord::new(5000).unwrap(),
        Coord::new(5000).unwrap(),
    ]));
    group.throughput(criterion::Throughput::Elements(125)); // 5^3
    group.bench_function("res3_proximity_r2", |b| {
        b.iter(|| {
            let p: Vec<_> = cr3.proximity(2).collect();
            black_box(p.len());
        })
    });

    group.finish();
}

// Spatial/cubedist/metric
//   Distance metric throughput for CoordCube
fn bench_coordcube_distance_metrics(c: &mut Criterion) {
    use tagma_core::{Coord, CoordCube, CoordPath};
    use tagma_geo::spatial::DistanceMetrics;

    let c1 = CoordCube::<3, 3, 1>::from_path(CoordPath::new([
        Coord::new(1000).unwrap(),
        Coord::new(2000).unwrap(),
        Coord::new(3000).unwrap(),
    ]));
    let c2 = CoordCube::<3, 3, 1>::from_path(CoordPath::new([
        Coord::new(1500).unwrap(),
        Coord::new(2500).unwrap(),
        Coord::new(3500).unwrap(),
    ]));

    let mut group = c.benchmark_group("Spatial/cubedist");

    group.bench_function("hamming", |bench| {
        bench.iter(|| {
            black_box(c1.hamming_distance(&c2));
        })
    });

    group.bench_function("euclidean", |bench| {
        bench.iter(|| {
            black_box(c1.euclidean_distance_approx(&c2));
        })
    });

    group.bench_function("manhattan", |bench| {
        bench.iter(|| {
            black_box(c1.manhattan_distance(&c2));
        })
    });

    // Multi-syllable (R=2) variants
    let a_r2 = CoordCube::<4, 2, 2>::from_path(CoordPath::new([
        Coord::new(1000).unwrap(),
        Coord::new(2000).unwrap(),
        Coord::new(3000).unwrap(),
        Coord::new(4000).unwrap(),
    ]));
    let b_r2 = CoordCube::<4, 2, 2>::from_path(CoordPath::new([
        Coord::new(1500).unwrap(),
        Coord::new(2500).unwrap(),
        Coord::new(3500).unwrap(),
        Coord::new(4500).unwrap(),
    ]));

    group.bench_function("hamming_r2", |bench| {
        bench.iter(|| {
            black_box(a_r2.hamming_distance(&b_r2));
        })
    });
    group.bench_function("euclidean_r2", |bench| {
        bench.iter(|| {
            black_box(a_r2.euclidean_distance_approx(&b_r2));
        })
    });
    group.bench_function("manhattan_r2", |bench| {
        bench.iter(|| {
            black_box(a_r2.manhattan_distance(&b_r2));
        })
    });

    group.finish();
}

// Spatial/kvproximity/store_density
//   End-to-end KV spatial query throughput with different store densities
fn bench_kv_spatial_proximity(c: &mut Criterion) {
    use tagma_core::{Coord, CoordCube, CoordPath};
    use tagma_geo::spatial::SpatialOps;
    use tagma_kv::coord_gen::CoordKey;
    use tagma_kv::coord_kv_n::CoordKVN;
    use tagma_kv::spatial::CoordCubeKV;
    use tagma_kv::CoordKVKey;

    let mut group = c.benchmark_group("Spatial/kvproximity");

    // Dense store: fill 100x100 region around center
    let mut kv = CoordKVN::<2>::new();
    let center = CoordPath::<2>::new([Coord::new(5000).unwrap(), Coord::new(5000).unwrap()]);
    let fill_box = CoordCube::<2, 2, 1>::from_path(center);
    let fill_ranges = [(4950u16, 5050u16), (4950u16, 5050u16)];
    for path in fill_box.bounding_box(&fill_ranges) {
        let key = CoordKey::from_coord_path(&path);
        kv.insert_by_coordkey(&key, b"v".to_vec());
    }

    let query_center = CoordPath::<2>::new([Coord::new(5000).unwrap(), Coord::new(5000).unwrap()]);

    group.throughput(criterion::Throughput::Elements(9)); // 3^2
    group.bench_function("dense_r1_proximity", |b| {
        b.iter(|| {
            let r = kv.proximity::<2, 1>(&query_center, 1);
            black_box(r.len());
        })
    });

    group.throughput(criterion::Throughput::Elements(25)); // 5^2
    group.bench_function("dense_r2_proximity", |b| {
        b.iter(|| {
            let r = kv.proximity::<2, 1>(&query_center, 2);
            black_box(r.len());
        })
    });

    // Sparse store: only a few scattered entries
    let mut sparse_kv = CoordKVN::<2>::new();
    for p in [4950u16, 5000u16, 5050u16] {
        for q in [4950u16, 5000u16, 5050u16] {
            let key = CoordKey::new([p as u8, q as u8]);
            sparse_kv.insert_by_coordkey(&key, b"v".to_vec());
        }
    }

    group.throughput(criterion::Throughput::Elements(9));
    group.bench_function("sparse_r1_proximity", |b| {
        b.iter(|| {
            let r = sparse_kv.proximity::<2, 1>(&query_center, 1);
            black_box(r.len());
        })
    });

    // Empty store
    let empty_kv: CoordKVN<2> = CoordKVN::new();
    group.bench_function("empty_r1_proximity", |b| {
        b.iter(|| {
            let r = empty_kv.proximity::<2, 1>(&query_center, 1);
            black_box(r.len());
        })
    });

    group.finish();
}

// Spatial/cubebox/n_scaling
//   Bounding box enumeration throughput vs N (number of syllables)
fn bench_coordcube_box_n_scaling(c: &mut Criterion) {
    use tagma_core::{Coord, CoordCube, CoordPath};
    use tagma_geo::spatial::SpatialOps;

    let mid = 5000u16;
    let mut group = c.benchmark_group("Spatial/cubebox");

    // N=2, D=2, R=1: 2D box 10x10 = 100 paths
    let c2 = CoordCube::<2, 2, 1>::from_path(CoordPath::new([
        Coord::new(mid).unwrap(),
        Coord::new(mid).unwrap(),
    ]));
    group.throughput(criterion::Throughput::Elements(100));
    group.bench_function("n2_box_10x10", |b| {
        let r = [(0u16, 9u16), (0u16, 9u16)];
        b.iter(|| {
            let p: Vec<_> = c2.bounding_box(&r).collect();
            black_box(p.len());
        })
    });

    // N=4, D=4, R=1: 4D box 4x4x4x4 = 256 paths
    let c4 = CoordCube::<4, 4, 1>::from_path(CoordPath::new([
        Coord::new(mid).unwrap(),
        Coord::new(mid).unwrap(),
        Coord::new(mid).unwrap(),
        Coord::new(mid).unwrap(),
    ]));
    group.throughput(criterion::Throughput::Elements(256));
    group.bench_function("n4_box_4x4x4x4", |b| {
        let r = [(0u16, 3u16), (0u16, 3u16), (0u16, 3u16), (0u16, 3u16)];
        b.iter(|| {
            let p: Vec<_> = c4.bounding_box(&r).collect();
            black_box(p.len());
        })
    });

    // N=6, D=6, R=1: 6D box 3^6 = 729 paths
    let c6 = CoordCube::<6, 6, 1>::from_path(CoordPath::new([
        Coord::new(mid).unwrap(),
        Coord::new(mid).unwrap(),
        Coord::new(mid).unwrap(),
        Coord::new(mid).unwrap(),
        Coord::new(mid).unwrap(),
        Coord::new(mid).unwrap(),
    ]));
    group.throughput(criterion::Throughput::Elements(729));
    group.bench_function("n6_box_3x3x3x3x3x3", |b| {
        let r = [(0u16, 2u16); 6];
        b.iter(|| {
            let p: Vec<_> = c6.bounding_box(&r).collect();
            black_box(p.len());
        })
    });

    group.finish();
}

// Spatial/cubeoverhead/axis_vs_raw
//   Overhead of CoordCube axis extraction vs raw CoordPath access
fn bench_coordcube_overhead(c: &mut Criterion) {
    use tagma_core::{Coord, CoordCube, CoordPath};

    let path = CoordPath::<6>::new([
        Coord::new(0).unwrap(),
        Coord::new(1).unwrap(),
        Coord::new(2).unwrap(),
        Coord::new(3).unwrap(),
        Coord::new(4).unwrap(),
        Coord::new(5).unwrap(),
    ]);
    let cube = CoordCube::<6, 3, 2>::from_path(path);

    let mut group = c.benchmark_group("Spatial/cubeoverhead");

    group.bench_function("raw_path_get_3x", |b| {
        b.iter(|| {
            let _a = CoordPath::new([
                path.coords()[0],
                path.coords()[1],
                path.coords()[2],
                path.coords()[3],
                path.coords()[4],
                path.coords()[5],
            ]);
        })
    });

    group.bench_function("cube_axis_3x", |b| {
        b.iter(|| {
            let _a = cube.axis(0);
            let _b = cube.axis(1);
            let _c = cube.axis(2);
        })
    });

    group.bench_function("cube_from_path", |b| {
        b.iter(|| {
            black_box(CoordCube::<6, 3, 2>::from_path(path));
        })
    });

    group.finish();
}

// ===========================================================================
// CoordCube: Tree+CoordPath vs Tree+CoordCube direct comparison
// ===========================================================================

// Spatial/cubevspath
//   Direct comparison: sequential CoordSpaceN2 lookups vs CoordCube proximity
//   on the same 10K-entry tree store (9 keys in r=1 neighborhood).
//   Tree+Path: 9 sequential lookups = 196 ns (21.8 ns each)
//   Tree+Cube: proximity r=1 = 285 ns (gen 15 ns + 9 lookups 196 ns + push 74 ns)
//   Sparse store: Tree+Cube = 48 ns (vs 196 ns, 4.1x faster)
fn bench_coordcube_path_vs_cube(c: &mut Criterion) {
    use tagma_core::{Coord, CoordCube, CoordPath};
    use tagma_geo::spatial::SpatialOps;
    use tagma_kv::coord_gen::CoordKey;
    use tagma_kv::coord_kv_n::CoordKVN;
    use tagma_kv::spatial::CoordCubeKV;
    use tagma_kv::CoordKVKey;

    let mut group = c.benchmark_group("Spatial/cubevspath");

    // Dense tree store: 10K entries in a 100x100 region
    let mut kv = CoordKVN::<2>::new();
    let center_path = CoordPath::<2>::new([Coord::new(5000).unwrap(), Coord::new(5000).unwrap()]);
    let fill_box = CoordCube::<2, 2, 1>::from_path(center_path);
    let fill_ranges = [(4950u16, 5050u16), (4950u16, 5050u16)];
    for path in fill_box.bounding_box(&fill_ranges) {
        let key = CoordKey::from_coord_path(&path);
        kv.insert_by_coordkey(&key, b"v".to_vec());
    }

    // Pre-compute the 9 neighbor keys for sequential lookup
    let neighbor_paths: Vec<_> = fill_box.proximity(1).collect();
    let neighbor_keys: Vec<_> = neighbor_paths
        .iter()
        .map(CoordKey::from_coord_path)
        .collect();

    let query_center = CoordPath::<2>::new([Coord::new(5000).unwrap(), Coord::new(5000).unwrap()]);

    // Tree+CoordPath: 9 sequential lookups
    group.bench_function("tree_path_sequential_9", |b| {
        b.iter(|| {
            let mut count = 0usize;
            for key in &neighbor_keys {
                if kv.get_by_coordkey(key).is_some() {
                    count += 1;
                }
            }
            black_box(count);
        })
    });

    // Tree+CoordCube: proximity r=1 (generates 9 paths, looks them up)
    group.bench_function("tree_cube_proximity_r1", |b| {
        b.iter(|| {
            let r = kv.proximity::<2, 1>(&query_center, 1);
            black_box(r.len());
        })
    });

    // Tree+CoordCube: proximity r=2 (25 paths)
    group.bench_function("tree_cube_proximity_r2", |b| {
        b.iter(|| {
            let r = kv.proximity::<2, 1>(&query_center, 2);
            black_box(r.len());
        })
    });

    group.finish();
}

// ===========================================================================
// CoordCube: large-N tree, hierarchical query, proximity_hamming
// ===========================================================================

// Spatial/cubelargen
//   CoordCube on large-N trees (N=12, N=19). Tests that spatial queries
//   work on deep trees with small radii to keep output bounded.
//   D=12,R=1 (N=12): 12 dimensions, r=0 → 1 path, r=1 → 3^12 paths
//   D=19,R=1 (N=19): 19 dimensions, r=0 → 1 path only (r=1 would be 3^19)
fn bench_coordcube_large_n(c: &mut Criterion) {
    use tagma_core::{Coord, CoordCube, CoordPath};
    use tagma_geo::spatial::SpatialOps;
    use tagma_kv::coord_gen::CoordKey;
    use tagma_kv::coord_kv_n::CoordKVN;
    use tagma_kv::spatial::CoordCubeKV;
    use tagma_kv::CoordKVKey;

    let mut group = c.benchmark_group("Spatial/cubelargen");

    // N=12, D=12, R=1: path gen r=0 on tree
    {
        let path = CoordPath::<12>::new(core::array::from_fn(|i| {
            Coord::new((i as u16) % 11172).unwrap()
        }));
        let cube = CoordCube::<12, 12, 1>::from_path(path);
        group.bench_function("n12_path_gen_r0", |b| {
            b.iter(|| black_box(cube.proximity(0).count()));
        });
    }

    // N=12, D=12, R=1: KV proximity r=0 on tree
    {
        let mut kv = CoordKVN::<12>::new();
        let path = CoordPath::<12>::new(core::array::from_fn(|i| {
            Coord::new((i as u16) % 11172).unwrap()
        }));
        let key = CoordKey::from_coord_path(&path);
        kv.insert_by_coordkey(&key, b"v".to_vec());
        let center = CoordPath::<12>::new(core::array::from_fn(|i| {
            Coord::new((i as u16) % 11172).unwrap()
        }));
        group.bench_function("n12_kv_proximity_r0", |b| {
            b.iter(|| {
                let r = kv.proximity::<12, 1>(&center, 0);
                black_box(r.len());
            })
        });
    }

    // N=19, D=19, R=1: path gen r=0 (single path, 19-D coord)
    {
        let path = CoordPath::<19>::new(core::array::from_fn(|i| {
            Coord::new(Coord::N_VALID as u16 - 1 - (i as u16 % 11172)).unwrap()
        }));
        let cube = CoordCube::<19, 19, 1>::from_path(path);
        group.bench_function("n19_path_gen_r0", |b| {
            b.iter(|| black_box(cube.proximity(0).count()));
        });
    }

    // N=19, D=19, R=1: KV proximity r=0 on tree
    {
        let mut kv = CoordKVN::<19>::new();
        let path = CoordPath::<19>::new(core::array::from_fn(|i| {
            Coord::new((i as u16) % 11172).unwrap()
        }));
        let key = CoordKey::from_coord_path(&path);
        kv.insert_by_coordkey(&key, b"v".to_vec());
        let center = CoordPath::<19>::new(core::array::from_fn(|i| {
            Coord::new((i as u16) % 11172).unwrap()
        }));
        group.bench_function("n19_kv_proximity_r0", |b| {
            b.iter(|| {
                let r = kv.proximity::<19, 1>(&center, 0);
                black_box(r.len());
            })
        });
    }

    group.finish();
}

// Spatial/hierarchical
//   Two-phase query: coarse CoordCube filter → fine CoordPath match.
//   Phase 1: CoordCube<2,2> (D=2,R=2, N=4) generates candidates at low res.
//   Phase 2: CoordPath filters generated paths against stored keys.
fn bench_coordcube_hierarchical(c: &mut Criterion) {
    use tagma_core::{Coord, CoordCube, CoordPath};
    use tagma_geo::spatial::SpatialOps;
    use tagma_kv::coord_gen::CoordKey;
    use tagma_kv::coord_kv_n::CoordKVN;
    use tagma_kv::spatial::CoordCubeKV;
    use tagma_kv::CoordKVKey;

    let mut group = c.benchmark_group("Spatial/hierarchical");

    // Fill a 4-syllable store (D=2, R=2 → N=4)
    let mut kv = CoordKVN::<4>::new();
    let center = CoordPath::<4>::new([
        Coord::new(5000).unwrap(),
        Coord::new(5000).unwrap(),
        Coord::new(5000).unwrap(),
        Coord::new(5000).unwrap(),
    ]);
    for d0 in [4995u16, 5000u16, 5005u16] {
        for d1 in [4995u16, 5000u16, 5005u16] {
            let p = CoordPath::<4>::new([
                Coord::new(d0).unwrap(),
                Coord::new(d0 + 1).unwrap(),
                Coord::new(d1).unwrap(),
                Coord::new(d1 + 1).unwrap(),
            ]);
            kv.insert_by_coordkey(&CoordKey::from_coord_path(&p), b"v".to_vec());
        }
    }

    // CoordCube<4, 2, 2>: N=4, D=2 dimensions with R=2 syllables each.
    // proximity() creates ranges per-syllable (4 ranges for 4 syllables),
    // then BoundingBoxIter enumerates all combinations — this correctly
    // models a 2D L∞ neighborhood where each dimension has 2 syllables.
    let cube = CoordCube::<4, 2, 2>::from_path(center);

    group.bench_function("proximity_r1_2phase", |b| {
        b.iter(|| {
            // Phase 1: generate candidates via CoordCube proximity
            let candidates: Vec<_> = cube.proximity(1).collect();
            // Phase 2: filter by exact store lookup
            let mut count = 0usize;
            for path in &candidates {
                let key = CoordKey::from_coord_path(path);
                if kv.get_by_coordkey(&key).is_some() {
                    count += 1;
                }
            }
            black_box(count);
        })
    });

    // Compare with direct KV proximity
    group.bench_function("kv_proximity_r1_direct", |b| {
        b.iter(|| {
            let r = kv.proximity::<4, 1>(&center, 1);
            black_box(r.len());
        })
    });

    group.finish();
}

// Spatial/cubehamming
//   Compare proximity() vs proximity_hamming() at same radius.
//   proximity_hamming filters L∞ neighbors by Hamming distance.
fn bench_coordcube_proximity_hamming(c: &mut Criterion) {
    use tagma_core::{Coord, CoordCube, CoordPath};
    use tagma_geo::spatial::SpatialOps;

    let path = CoordPath::<3>::new([
        Coord::new(5000).unwrap(),
        Coord::new(5000).unwrap(),
        Coord::new(5000).unwrap(),
    ]);
    let cube = CoordCube::<3, 3, 1>::from_path(path);

    let mut group = c.benchmark_group("Spatial/cubehamming");

    group.bench_function("proximity_r1", |b| {
        b.iter(|| {
            let p: Vec<_> = cube.proximity(1).collect();
            black_box(p.len());
        })
    });

    group.bench_function("proximity_hamming_r1", |b| {
        b.iter(|| {
            let p: Vec<_> = cube.proximity_hamming(1).collect();
            black_box(p.len());
        })
    });

    group.bench_function("proximity_r3", |b| {
        b.iter(|| {
            let p: Vec<_> = cube.proximity(3).collect();
            black_box(p.len());
        })
    });

    group.bench_function("proximity_hamming_r3", |b| {
        b.iter(|| {
            let p: Vec<_> = cube.proximity_hamming(3).collect();
            black_box(p.len());
        })
    });

    group.finish();
}

// N_scaling/get  (single lookup, ARMv8.4-A Firestorm)
//   N=1   CoordSpace      0.39 ns   space 10^4  (inline stack array, no alloc)
//   N=2   CoordSpace2     0.39 ns   space 10^8  (dense heap, 2.4x faster)
//   N=2   CoordSpaceN2    0.94 ns   space 10^8  (tree)
//   N=3   CoordSpaceN3    2.69 ns   space 10^12
//   N=3   CoordSpaceM3    0.40 ns   space 10^12 (mmap dense, 6.7x faster)
//   N=6   CoordSpaceN6    5.91 ns   space 10^24
//   N=12  CoordSpaceN12   23.3  ns   space 10^67
//   N=19  CoordSpaceN19   58.6  ns   space 10^77 (SHA-256 scale)
fn bench_n_scaling_get(c: &mut Criterion) {
    let path6 = tagma_core::CoordPath::<6>::new(core::array::from_fn(|i| {
        tagma_core::Coord::new(i as u16).unwrap()
    }));
    let path12 = tagma_core::CoordPath::<12>::new(core::array::from_fn(|i| {
        tagma_core::Coord::new(i as u16).unwrap()
    }));
    let path19 = tagma_core::CoordPath::<19>::new(core::array::from_fn(|i| {
        tagma_core::Coord::new(i as u16).unwrap()
    }));
    let path2 = tagma_core::CoordPath::<2>::new([
        tagma_core::Coord::new(0).unwrap(),
        tagma_core::Coord::new(1).unwrap(),
    ]);
    let path3 = tagma_core::CoordPath::<3>::new([
        tagma_core::Coord::new(0).unwrap(),
        tagma_core::Coord::new(1).unwrap(),
        tagma_core::Coord::new(2).unwrap(),
    ]);

    {
        let mut cs = tagma_core::CoordSpace::new();
        cs.place(tagma_core::Coord::new(0).unwrap(), 42u64);
        let mut group = c.benchmark_group("N_scaling/get/N=1");
        group.bench_function("CoordSpace", |b| {
            b.iter(|| black_box(cs.at(&tagma_core::Coord::new(0).unwrap())))
        });
        group.finish();
    }
    {
        let mut cs = tagma_core::CoordSpaceN2::<u64>::new();
        cs.place_path(&path2, 42);
        let mut group = c.benchmark_group("N_scaling/get/N=2");
        group.bench_function("CoordSpaceN2", |b| b.iter(|| black_box(cs.at_path(&path2))));
        group.finish();
    }
    {
        let mut cs = tagma_core::CoordSpace2::<u64>::new();
        cs.place_path(&path2, 42);
        let mut group = c.benchmark_group("N_scaling/get/N=2");
        group.bench_function("CoordSpace2", |b| b.iter(|| black_box(cs.at_path(&path2))));
        group.finish();
    }
    {
        let mut cs = tagma_core::CoordSpaceN3::<u64>::new();
        cs.place_path(&path3, 42);
        let mut group = c.benchmark_group("N_scaling/get/N=3");
        group.bench_function("CoordSpaceN3", |b| b.iter(|| black_box(cs.at_path(&path3))));
        group.finish();
    }
    #[cfg(feature = "mmap")]
    {
        let mut cs = tagma_core::CoordSpaceM3::<u64>::new();
        cs.place_path(&path3, 42);
        let mut group = c.benchmark_group("N_scaling/get/N=3");
        group.bench_function("CoordSpaceM3", |b| b.iter(|| black_box(cs.at_path(&path3))));
        group.finish();
    }
    {
        let mut cs = tagma_core::CoordSpaceN6::<u64>::new();
        cs.place_path(&path6, 42);
        let mut group = c.benchmark_group("N_scaling/get/N=6");
        group.bench_function("CoordSpaceN6", |b| b.iter(|| black_box(cs.at_path(&path6))));
        group.finish();
    }
    {
        let mut cs = tagma_core::CoordSpaceN12::<u64>::new();
        cs.place_path(&path12, 42);
        let mut group = c.benchmark_group("N_scaling/get/N=12");
        group.bench_function("CoordSpaceN12", |b| {
            b.iter(|| black_box(cs.at_path(&path12)))
        });
        group.finish();
    }
    {
        let mut cs = tagma_core::CoordSpaceN19::<u64>::new();
        cs.place_path(&path19, 42);
        let mut group = c.benchmark_group("N_scaling/get/N=19");
        group.bench_function("CoordSpaceN19", |b| {
            b.iter(|| black_box(cs.at_path(&path19)))
        });
        group.finish();
    }
}

// ===========================================================================
// CoordSpaceN2 (N=2) benchmarks — cross-product FIH-like scenario
// ===========================================================================

// Dense vs tree comparison at N=2 (ARMv8.4-A Firestorm, CoordSpace2 vs CoordSpaceN2):
//   insert/1000   CoordSpace2   155 µs   CoordSpaceN2   841 µs    5.4x faster
//   get/1000      CoordSpace2  0.39 µs   CoordSpaceN2  1.15 µs    2.9x faster
//   single get    CoordSpace2  0.39 ns   CoordSpaceN2  0.90 ns    2.3x faster

// CoordSpaceN2/insert/1000          841 µs
// CoordSpace2/insert/1000           155 µs  (dense array)
fn bench_cm2_insert_1000(c: &mut Criterion) {
    let mut group = c.benchmark_group("N=2/insert/1000");
    group.bench_function("CoordSpaceN2", |b| {
        b.iter(|| {
            let mut map = tagma_core::CoordSpaceN2::new();
            for i in 0u16..100 {
                for j in 0u16..10 {
                    let path = tagma_core::CoordPath::new([
                        tagma_core::Coord::new(i).unwrap(),
                        tagma_core::Coord::new(j).unwrap(),
                    ]);
                    black_box(map.place_path(&path, (i * 100 + j) as u32));
                }
            }
            black_box(map);
        })
    });
    group.bench_function("CoordSpace2", |b| {
        b.iter(|| {
            let mut map = tagma_core::CoordSpace2::new();
            for i in 0u16..100 {
                for j in 0u16..10 {
                    let path = tagma_core::CoordPath::new([
                        tagma_core::Coord::new(i).unwrap(),
                        tagma_core::Coord::new(j).unwrap(),
                    ]);
                    black_box(map.place_path(&path, (i * 100 + j) as u32));
                }
            }
            black_box(map);
        })
    });
    group.finish();
}

// CoordSpaceN2/get/1000             4.92 µs
// CoordSpace2/get/1000                 ?  µs  (dense array)
fn bench_cm2_get_1000(c: &mut Criterion) {
    let map_n2 = {
        let mut map = tagma_core::CoordSpaceN2::new();
        for i in 0u16..100 {
            for j in 0u16..10 {
                let path = tagma_core::CoordPath::new([
                    tagma_core::Coord::new(i).unwrap(),
                    tagma_core::Coord::new(j).unwrap(),
                ]);
                map.place_path(&path, (i * 100 + j) as u32);
            }
        }
        map
    };
    let map_dense = {
        let mut map = tagma_core::CoordSpace2::new();
        for i in 0u16..100 {
            for j in 0u16..10 {
                let path = tagma_core::CoordPath::new([
                    tagma_core::Coord::new(i).unwrap(),
                    tagma_core::Coord::new(j).unwrap(),
                ]);
                map.place_path(&path, (i * 100 + j) as u32);
            }
        }
        map
    };
    let mut group = c.benchmark_group("N=2/get/1000");
    group.bench_function("CoordSpaceN2", |b| {
        b.iter(|| {
            for i in 0u16..100 {
                for j in 0u16..10 {
                    let path = tagma_core::CoordPath::new([
                        tagma_core::Coord::new(i).unwrap(),
                        tagma_core::Coord::new(j).unwrap(),
                    ]);
                    black_box(map_n2.at_path(&path));
                }
            }
        })
    });
    group.bench_function("CoordSpace2", |b| {
        b.iter(|| {
            for i in 0u16..100 {
                for j in 0u16..10 {
                    let path = tagma_core::CoordPath::new([
                        tagma_core::Coord::new(i).unwrap(),
                        tagma_core::Coord::new(j).unwrap(),
                    ]);
                    black_box(map_dense.at_path(&path));
                }
            }
        })
    });
    group.finish();
}

// ===========================================================================
// Group runner
// ===========================================================================

criterion_group!(
    name = inserts;
    config = Criterion::default();
    targets = bench_tagma_insert_all, bench_std_insert_all,
              bench_tagma_insert_random, bench_std_insert_random
);
criterion_group!(
    name = lookup;
    config = Criterion::default();
    targets = bench_tagma_get_all, bench_std_get_all,
              bench_tagma_overwrite_all, bench_std_overwrite_all
);
criterion_group!(
    name = mutate;
    config = Criterion::default();
    targets = bench_tagma_remove_all, bench_std_remove_all,
              bench_tagma_entry, bench_std_entry,
              bench_tagma_retain_half, bench_std_retain_half
);
criterion_group!(
    name = iterate;
    config = Criterion::default();
    targets = bench_tagma_iter, bench_std_iter,
              bench_tagma_drain_all, bench_std_drain_all,
              bench_baseline_iterate
);
criterion_group!(
    name = micro;
    config = Criterion::default().sample_size(200);
    targets = bench_tagma_get_single, bench_std_get_single,
              bench_tagma_insert_single, bench_std_insert_single
);
criterion_group!(
    name = n_scaling;
    config = Criterion::default().sample_size(50);
    targets = bench_n_scaling_get
);
criterion_group!(
    name = n2_comparison;
    config = Criterion::default();
    targets = bench_cm2_insert_1000, bench_cm2_get_1000
);
criterion_group!(
    name = large;
    config = Criterion::default().sample_size(20).measurement_time(std::time::Duration::from_secs(5));
    targets = bench_cs2_bulk_100k
);
criterion_group!(
    name = edge;
    config = Criterion::default();
    targets = bench_cs2_sparse_10M, bench_cs2_md_axis_projection, bench_cs2_nonexistent_prefix,
              bench_cs6_sparse_1k, bench_cs6_nonexistent_prefix, bench_cs6_md_axis_projection
);
criterion_group!(
    name = deep;
    config = Criterion::default();
    targets = bench_cs19_sparse_100, bench_cs19_nonexistent_prefix
);
criterion_group!(
    name = stress;
    config = Criterion::default().sample_size(30).measurement_time(std::time::Duration::from_secs(10));
    targets = bench_tagma_mixed_500k, bench_std_mixed_500k
);

criterion_group!(
    name = spatial;
    config = Criterion::default();
    targets = bench_spatial_axis_filter,
              bench_spatial_axis_filter_range,
              bench_spatial_cs2_prefix_scan,
              bench_coordset_spatial_query,
              bench_coordcube_proximity_radius,
              bench_coordcube_bounding_box,
              bench_coordcube_dim_scaling,
              bench_coordcube_distance_metrics,
              bench_kv_spatial_proximity,
              bench_coordcube_box_n_scaling,
              bench_coordcube_overhead,
              bench_coordcube_path_vs_cube,
              bench_coordcube_large_n,
              bench_coordcube_hierarchical,
              bench_coordcube_proximity_hamming
);

// ===========================================================================
// tagma-kv: 3-way comparison — static CoordSpace2 vs dynamic DynCoordSpace
//            vs std HashMap for string-key KV workloads
// ===========================================================================
//
// Single-op results (ns):
//   kv/static/insert/single        16.9  ns
//   kv/dynamic/insert/single       29.0  ns
//   kv/hashmap/insert/single       47.6  ns
//   kv/static/get/single            1.07 ns
//   kv/dynamic/get/single           4.19 ns
//   kv/hashmap/get/single          13.4  ns
//
// Batch results:
//   kv/static/insert/short_1k      21.4 µs
//   kv/dynamic/insert/short_1k     42.6 µs
//   kv/hashmap/insert/short_1k     57.1 µs
//   kv/static/insert/medium_1k     21.4 µs
//   kv/dynamic/insert/medium_1k    73.6 µs
//   kv/hashmap/insert/medium_1k    57.6 µs
//
// Batch get (10k keys, per-key ns):
//   kv/batch-get/static             0.84 ns
//   kv/batch-get/dynamic           12.0  ns
//   kv/batch-get/hashmap           19.1  ns
//
// Wrapper single-op (ns):
//   DynCoordKV/insert/single             49.4 ns
//   CoordKV2/insert/single               18.2 ns
//   CoordKVN<2>/insert/single            19.2 ns
//   HashMap<String>/insert/single        44.7 ns
//   DynCoordKV/get/single                42.4 ns
//   CoordKV2/get/single                  21.5 ns
//   CoordKVN<2>/get/single               21.8 ns
//   HashMap<String>/get/single           24.0 ns
//
// Wrapper batch get (10k keys, total):
//   DynCoordKV                          541 µs
//   CoordKV2                           14.5 µs
//   CoordKVN<2>                        14.7 µs
//   HashMap<String>                    236 µs
//
// Scale workload (get, per-op ns):
//               10k      1M      10M
//   CoordKV2   22.0    21.4    21.5     (flat)
//   CoordKVN<2> 22.7   21.9    22.1     (flat)
//   DynCoordKV 55.8    57.4    60.6     (+7%)
//   HashMap    21.9    24.2    23.8     (+19%)
//
// Scale workload (contains, per-op ns):
//               10k      1M      10M
//   CoordKV2   22.3    21.6    21.6     (flat)
//   CoordKVN<2> 23.2   —        —
//   DynCoordKV 54.7    —        —
//   HashMap    13.2    19.9    19.9     (+53%)
//
// Scale workload (insert, 65k keys):
//   CoordKV2   227 ms   (119 MB alloc dominated)
//   CoordKVN<2> 10.4 ms (65536 keys)
//   DynCoordKV 416 ms   (65k trie nodes)
//   HashMap    9.2 ms
// ===========================================================================

fn kv_short_keys(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| format!("k{:05}", i % 100_000)) // "k00000" .. "k99999" (6 bytes each)
        .collect()
}

fn kv_medium_keys(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| format!("key_{:08}", i % 100_000_000)) // "key_00000000" .. "key_99999999"
        .collect()
}

fn kv_value() -> Vec<u8> {
    b"value_data_32_bytes_xxxxxxxx".to_vec()
}

fn kv_boxed(v: &[u8]) -> Box<[u8]> {
    v.to_vec().into_boxed_slice()
}

use tagma_core::CoordPath;
use tagma_kv::coord_gen::{ByteWise, CoordGen, Prefix};

// ── Single-operation microbenchmarks (pre-built containers) ────────────

fn bench_kv_single_insert_static(c: &mut Criterion) {
    let key = "k000";
    let path = Prefix::<2>.generate(key).unwrap();
    let cp = CoordPath::new([path[0], path[1]]);

    c.bench_function("kv/static/insert/single", |b| {
        let mut space: tagma_core::CoordSpace2<Box<[u8]>> = tagma_core::CoordSpace2::new();
        b.iter(|| {
            black_box(space.place_path(black_box(&cp), kv_boxed(&kv_value())));
        })
    });
}

fn bench_kv_single_insert_dyn(c: &mut Criterion) {
    let key = "k000";
    let path = ByteWise.generate(key).unwrap();

    c.bench_function("kv/dynamic/insert/single", |b| {
        let mut space: tagma_core::DynCoordSpace<Box<[u8]>> = tagma_core::DynCoordSpace::new();
        b.iter(|| {
            black_box(space.place(black_box(&path), kv_boxed(&kv_value())));
        })
    });
}

fn bench_kv_single_insert_hashmap(c: &mut Criterion) {
    let key = "k000";

    c.bench_function("kv/hashmap/insert/single", |b| {
        let mut map: std::collections::HashMap<String, Box<[u8]>> =
            std::collections::HashMap::new();
        b.iter(|| {
            black_box(map.insert(key.to_string(), kv_boxed(&kv_value())));
        })
    });
}

fn bench_kv_single_get_static(c: &mut Criterion) {
    let key = "k000";
    let path = Prefix::<2>.generate(key).unwrap();
    let cp = CoordPath::new([path[0], path[1]]);
    let mut space: tagma_core::CoordSpace2<Box<[u8]>> = tagma_core::CoordSpace2::new();
    space.place_path(&cp, kv_boxed(&kv_value()));

    c.bench_function("kv/static/get/single", |b| {
        b.iter(|| {
            black_box(black_box(&space).at_path(black_box(&cp)));
        })
    });
}

fn bench_kv_single_get_dyn(c: &mut Criterion) {
    let key = "k000";
    let path = ByteWise.generate(key).unwrap();
    let mut space: tagma_core::DynCoordSpace<Box<[u8]>> = tagma_core::DynCoordSpace::new();
    space.place(&path, kv_boxed(&kv_value()));

    c.bench_function("kv/dynamic/get/single", |b| {
        b.iter(|| {
            black_box(black_box(&space).at(black_box(&path)));
        })
    });
}

fn bench_kv_single_get_hashmap(c: &mut Criterion) {
    let key = "k000";
    let mut map: std::collections::HashMap<String, Box<[u8]>> = std::collections::HashMap::new();
    map.insert(key.to_string(), kv_boxed(&kv_value()));

    c.bench_function("kv/hashmap/get/single", |b| {
        b.iter(|| {
            black_box(black_box(&map).get(key));
        })
    });
}

// ── Batch insert: 1,000 short keys (4 bytes) ───────────────────────────

fn bench_kv_batch_insert_static_short(c: &mut Criterion) {
    let keys: Vec<(CoordPath<2>, Box<[u8]>)> = kv_short_keys(1000)
        .iter()
        .map(|k| {
            let path = Prefix::<2>.generate(k).unwrap();
            let cp = CoordPath::new([path[0], path[1]]);
            (cp, kv_boxed(&kv_value()))
        })
        .collect();

    c.bench_function("kv/static/insert/short_1k", |b| {
        let mut space: tagma_core::CoordSpace2<Box<[u8]>> = tagma_core::CoordSpace2::new();
        b.iter(|| {
            for (cp, val) in &keys {
                black_box(space.place_path(cp, kv_boxed(val)));
            }
            black_box(&space);
        })
    });
}

fn bench_kv_batch_insert_dyn_short(c: &mut Criterion) {
    let keys: Vec<(Vec<tagma_core::Coord>, Box<[u8]>)> = kv_short_keys(1000)
        .iter()
        .map(|k| {
            let path = ByteWise.generate(k).unwrap();
            (path, kv_boxed(&kv_value()))
        })
        .collect();

    c.bench_function("kv/dynamic/insert/short_1k", |b| {
        let mut space: tagma_core::DynCoordSpace<Box<[u8]>> = tagma_core::DynCoordSpace::new();
        b.iter(|| {
            for (path, val) in &keys {
                black_box(space.place(path, kv_boxed(val)));
            }
            black_box(&space);
        })
    });
}

fn bench_kv_batch_insert_hashmap_short(c: &mut Criterion) {
    let keys: Vec<(String, Box<[u8]>)> = kv_short_keys(1000)
        .iter()
        .map(|k| (k.clone(), kv_boxed(&kv_value())))
        .collect();

    c.bench_function("kv/hashmap/insert/short_1k", |b| {
        let mut map: std::collections::HashMap<String, Box<[u8]>> =
            std::collections::HashMap::new();
        b.iter(|| {
            for (k, val) in &keys {
                black_box(map.insert(k.clone(), kv_boxed(val)));
            }
            black_box(&map);
        })
    });
}

// ── Batch insert: 1,000 medium keys (14 bytes) ─────────────────────────

fn bench_kv_batch_insert_static_medium(c: &mut Criterion) {
    let keys: Vec<(CoordPath<2>, Box<[u8]>)> = kv_medium_keys(1000)
        .iter()
        .map(|k| {
            let path = Prefix::<2>.generate(k).unwrap();
            let cp = CoordPath::new([path[0], path[1]]);
            (cp, kv_boxed(&kv_value()))
        })
        .collect();

    c.bench_function("kv/static/insert/medium_1k", |b| {
        let mut space: tagma_core::CoordSpace2<Box<[u8]>> = tagma_core::CoordSpace2::new();
        b.iter(|| {
            for (cp, val) in &keys {
                black_box(space.place_path(cp, kv_boxed(val)));
            }
            black_box(&space);
        })
    });
}

fn bench_kv_batch_insert_dyn_medium(c: &mut Criterion) {
    let keys: Vec<(Vec<tagma_core::Coord>, Box<[u8]>)> = kv_medium_keys(1000)
        .iter()
        .map(|k| {
            let path = ByteWise.generate(k).unwrap();
            (path, kv_boxed(&kv_value()))
        })
        .collect();

    c.bench_function("kv/dynamic/insert/medium_1k", |b| {
        let mut space: tagma_core::DynCoordSpace<Box<[u8]>> = tagma_core::DynCoordSpace::new();
        b.iter(|| {
            for (path, val) in &keys {
                black_box(space.place(path, kv_boxed(val)));
            }
            black_box(&space);
        })
    });
}

fn bench_kv_batch_insert_hashmap_medium(c: &mut Criterion) {
    let keys: Vec<(String, Box<[u8]>)> = kv_medium_keys(1000)
        .iter()
        .map(|k| (k.clone(), kv_boxed(&kv_value())))
        .collect();

    c.bench_function("kv/hashmap/insert/medium_1k", |b| {
        let mut map: std::collections::HashMap<String, Box<[u8]>> =
            std::collections::HashMap::new();
        b.iter(|| {
            for (k, val) in &keys {
                black_box(map.insert(k.clone(), kv_boxed(val)));
            }
            black_box(&map);
        })
    });
}

// ── Batch get: all three variants in a single benchmark group ──────────

fn bench_kv_batch_get_all(c: &mut Criterion) {
    let short_keys = kv_short_keys(10_000);
    let mut group = c.benchmark_group("kv/batch-get/short_10k");

    // Static: pre-populate CoordSpace2
    {
        let mut space: tagma_core::CoordSpace2<Box<[u8]>> = tagma_core::CoordSpace2::new();
        let paths: Vec<CoordPath<2>> = short_keys
            .iter()
            .map(|k| {
                let p = Prefix::<2>.generate(k).unwrap();
                CoordPath::new([p[0], p[1]])
            })
            .collect();
        for cp in &paths {
            space.place_path(cp, kv_boxed(&kv_value()));
        }
        group.bench_function("static/Prefix2/CoordSpace2", |b| {
            b.iter(|| {
                for cp in &paths {
                    black_box(black_box(&space).at_path(cp));
                }
            })
        });
    }

    // Dynamic: pre-populate DynCoordSpace
    {
        let mut space: tagma_core::DynCoordSpace<Box<[u8]>> = tagma_core::DynCoordSpace::new();
        let paths: Vec<Vec<tagma_core::Coord>> = short_keys
            .iter()
            .map(|k| ByteWise.generate(k).unwrap())
            .collect();
        for path in &paths {
            space.place(path, kv_boxed(&kv_value()));
        }
        group.bench_function("dynamic/ByteWise/DynCoordSpace", |b| {
            b.iter(|| {
                for path in &paths {
                    black_box(black_box(&space).at(path));
                }
            })
        });
    }

    // HashMap: pre-populate
    {
        let mut map: std::collections::HashMap<String, Box<[u8]>> =
            std::collections::HashMap::new();
        for k in &short_keys {
            map.insert(k.clone(), kv_boxed(&kv_value()));
        }
        group.bench_function("hashmap/String", |b| {
            b.iter(|| {
                for k in &short_keys {
                    black_box(black_box(&map).get(k));
                }
            })
        });
    }

    group.finish();
}

// ===========================================================================
// Wrapper-type benchmarks: DynCoordKV vs CoordKV2 vs CoordKVN vs HashMap
// through the CoordKV trait API (insert/get via &str)
// ===========================================================================

use tagma_kv::coord_kv_n::CoordKVN;
use tagma_kv::{CoordKV, CoordKV2, CoordKVKey, CoordKey, DynCoordKV};

fn kv_2byte_keys(count: usize) -> Vec<String> {
    // 2-char alphanumeric keys: "00", "01", ... "zz" (1296 unique from 36^2)
    let chars: Vec<char> = "0123456789abcdefghijklmnopqrstuvwxyz".chars().collect();
    let base = chars.len();
    (0..count)
        .map(|i| {
            let idx = i % (base * base);
            let hi = idx / base;
            let lo = idx % base;
            format!("{}{}", chars[hi], chars[lo])
        })
        .collect()
}

fn kv_2byte_all() -> Vec<CoordKey<2>> {
    // All 65536 possible 2-byte CoordKey<2> values
    let mut keys = Vec::with_capacity(65536);
    for hi in 0u8..=255 {
        for lo in 0u8..=255 {
            keys.push(CoordKey::new([hi, lo]));
        }
    }
    keys
}

fn bench_kv_wrapper_single_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("tagma-kv-wrapper/insert/single");

    group.bench_function("DynCoordKV", |b| {
        let mut kv = DynCoordKV::new();
        b.iter(|| {
            black_box(kv.insert("k000", kv_value()));
        })
    });

    group.bench_function("CoordKV2", |b| {
        let mut kv = CoordKV2::new();
        b.iter(|| {
            black_box(kv.insert("k0", kv_value()));
        })
    });

    group.bench_function("CoordKVN<2>", |b| {
        let mut kv = CoordKVN::<2>::new();
        b.iter(|| {
            black_box(kv.insert("k0", kv_value()));
        })
    });

    group.bench_function("HashMap<String>", |b| {
        let mut map: std::collections::HashMap<String, Vec<u8>> = std::collections::HashMap::new();
        b.iter(|| {
            black_box(map.insert("k0".to_string(), kv_value()));
        })
    });

    group.finish();
}

// ── Single get ───────────────────────────────────────────────────────────

fn bench_kv_wrapper_single_get(c: &mut Criterion) {
    let mut group = c.benchmark_group("tagma-kv-wrapper/get/single");

    group.bench_function("DynCoordKV", |b| {
        let mut kv = DynCoordKV::new();
        kv.insert("k000", kv_value());
        b.iter(|| {
            black_box(kv.get("k000"));
        })
    });

    group.bench_function("CoordKV2", |b| {
        let mut kv = CoordKV2::new();
        kv.insert("k0", kv_value());
        b.iter(|| {
            black_box(kv.get("k0"));
        })
    });

    group.bench_function("CoordKVN<2>", |b| {
        let mut kv = CoordKVN::<2>::new();
        kv.insert("k0", kv_value());
        b.iter(|| {
            black_box(kv.get("k0"));
        })
    });

    group.bench_function("HashMap<String>", |b| {
        let mut map: std::collections::HashMap<String, Vec<u8>> = std::collections::HashMap::new();
        map.insert("k0".to_string(), kv_value());
        b.iter(|| {
            black_box(map.get("k0"));
        })
    });

    group.finish();
}

// ── Batch get: 1,000 short keys, all five stores ───────────────────────

fn bench_kv_wrapper_batch_get(c: &mut Criterion) {
    let short_keys = kv_short_keys(10_000);
    let byte2_keys = kv_2byte_keys(676);
    let val = kv_value();
    let mut group = c.benchmark_group("tagma-kv-wrapper/batch-get/10k");

    group.bench_function("DynCoordKV", |b| {
        let mut kv = DynCoordKV::new();
        for k in &short_keys {
            kv.insert(k, val.clone());
        }
        b.iter(|| {
            for k in &short_keys {
                black_box(kv.get(k));
            }
        })
    });

    group.bench_function("CoordKV2", |b| {
        let mut kv = CoordKV2::new();
        for k in &byte2_keys {
            kv.insert(k, val.clone());
        }
        b.iter(|| {
            for k in &byte2_keys {
                black_box(kv.get(k));
            }
        })
    });

    group.bench_function("CoordKVN<2>", |b| {
        let mut kv = CoordKVN::<2>::new();
        for k in &byte2_keys {
            kv.insert(k, val.clone());
        }
        b.iter(|| {
            for k in &byte2_keys {
                black_box(kv.get(k));
            }
        })
    });

    group.bench_function("HashMap<String>", |b| {
        let mut map: std::collections::HashMap<String, Vec<u8>> = std::collections::HashMap::new();
        for k in &short_keys {
            map.insert(k.clone(), val.clone());
        }
        b.iter(|| {
            for k in &short_keys {
                black_box(map.get(k));
            }
        })
    });

    group.finish();
}

// ── Three-scale workload: 10k, 1M, 10M ─────────────────────────────────
// kv_workload(name, unique_keys, cycles):
//   get/contains: unique_keys × cycles total operations
//   insert: unique_keys unique inserts
//
// Key count notes:
//   - DynCoordKV/HashMap: uses kv_short_keys (up to 100k unique 6-char keys)
//   - CoordKV2/CoordKVN<2>: str API limited to 1296 unique 2-char keys
//     (CoordKey<2> from str must be valid UTF-8, so only ASCII printable)
//   - For scales above 1296 unique 2-char keys, CoordKV2/CoordKVN<2> fall
//     back to by_coordkey API which accepts all 65536 byte pairs directly.

const STR_2BYTE_UNIQUE: usize = 1296; // 36^2 alphanumeric 2-char keys

fn kv_workload(c: &mut Criterion, scale: &str, n: usize, cycles: usize) {
    let keys = kv_short_keys(n);
    let k2_str = kv_2byte_keys(STR_2BYTE_UNIQUE); // 1296 unique 2-char str keys
    let all_k2 = kv_2byte_all(); // 65536 raw CoordKey<2>
    let val = kv_value();
    let group_name = format!("kv/workload/{}", scale);
    let mut group = c.benchmark_group(&group_name);

    // Use by_coordkey API when n exceeds the str-usable key limit
    let use_coordkey_api = n > STR_2BYTE_UNIQUE;

    // DynCoordKV
    {
        let mut kv = DynCoordKV::new();
        for k in &keys {
            kv.insert(k, val.clone());
        }
        group.bench_function("DynCoordKV/get", |b| {
            b.iter(|| {
                for _ in 0..cycles {
                    for k in &keys {
                        black_box(kv.get(k));
                    }
                }
            })
        });
        group.bench_function("DynCoordKV/contains", |b| {
            b.iter(|| {
                for _ in 0..cycles {
                    for k in &keys {
                        black_box(kv.contains_key(k));
                    }
                }
            })
        });
        group.bench_function("DynCoordKV/insert", |b| {
            b.iter(|| {
                let mut kv = DynCoordKV::new();
                for k in &keys {
                    kv.insert(k, val.clone());
                }
            })
        });
    }

    // CoordKV2 — str API for small scale, by_coordkey for 1M/10M
    {
        if !use_coordkey_api {
            let mut kv = CoordKV2::new();
            for k in &k2_str {
                kv.insert(k, val.clone());
            }
            group.bench_function("CoordKV2/get", |b| {
                b.iter(|| {
                    for _ in 0..cycles {
                        for k in &k2_str {
                            black_box(kv.get(k));
                        }
                    }
                })
            });
            group.bench_function("CoordKV2/contains", |b| {
                b.iter(|| {
                    for _ in 0..cycles {
                        for k in &k2_str {
                            black_box(kv.contains_key(k));
                        }
                    }
                })
            });
            group.bench_function("CoordKV2/insert", |b| {
                b.iter(|| {
                    let mut kv = CoordKV2::new();
                    for k in &k2_str {
                        kv.insert(k, val.clone());
                    }
                })
            });
        } else {
            let mut kv = CoordKV2::new();
            for ck in &all_k2 {
                kv.insert_by_coordkey(ck, val.clone());
            }
            group.bench_function("CoordKV2/get", |b| {
                b.iter(|| {
                    for _ in 0..cycles {
                        for ck in &all_k2 {
                            black_box(kv.get_by_coordkey(ck));
                        }
                    }
                })
            });
            group.bench_function("CoordKV2/contains", |b| {
                b.iter(|| {
                    for _ in 0..cycles {
                        for ck in &all_k2 {
                            black_box(kv.contains_key_by_coordkey(ck));
                        }
                    }
                })
            });
            group.bench_function("CoordKV2/insert", |b| {
                b.iter(|| {
                    let mut kv = CoordKV2::new();
                    for ck in &all_k2 {
                        kv.insert_by_coordkey(ck, val.clone());
                    }
                })
            });
        }
    }

    // CoordKVN<2> — str API for small scale, by_coordkey for 1M/10M
    {
        if !use_coordkey_api {
            let mut kv = CoordKVN::<2>::new();
            for k in &k2_str {
                kv.insert(k, val.clone());
            }
            group.bench_function("CoordKVN<2>/get", |b| {
                b.iter(|| {
                    for _ in 0..cycles {
                        for k in &k2_str {
                            black_box(kv.get(k));
                        }
                    }
                })
            });
            group.bench_function("CoordKVN<2>/contains", |b| {
                b.iter(|| {
                    for _ in 0..cycles {
                        for k in &k2_str {
                            black_box(kv.contains_key(k));
                        }
                    }
                })
            });
            group.bench_function("CoordKVN<2>/insert", |b| {
                b.iter(|| {
                    let mut kv = CoordKVN::<2>::new();
                    for k in &k2_str {
                        kv.insert(k, val.clone());
                    }
                })
            });
        } else {
            let mut kv = CoordKVN::<2>::new();
            for ck in &all_k2 {
                kv.insert_by_coordkey(ck, val.clone());
            }
            group.bench_function("CoordKVN<2>/get", |b| {
                b.iter(|| {
                    for _ in 0..cycles {
                        for ck in &all_k2 {
                            black_box(kv.get_by_coordkey(ck));
                        }
                    }
                })
            });
            group.bench_function("CoordKVN<2>/contains", |b| {
                b.iter(|| {
                    for _ in 0..cycles {
                        for ck in &all_k2 {
                            black_box(kv.contains_key_by_coordkey(ck));
                        }
                    }
                })
            });
            group.bench_function("CoordKVN<2>/insert", |b| {
                b.iter(|| {
                    let mut kv = CoordKVN::<2>::new();
                    for ck in &all_k2 {
                        kv.insert_by_coordkey(ck, val.clone());
                    }
                })
            });
        }
    }

    // HashMap<String>
    {
        let mut hm: std::collections::HashMap<String, Vec<u8>> = std::collections::HashMap::new();
        for k in &keys {
            hm.insert(k.clone(), val.clone());
        }
        group.bench_function("HashMap<String>/get", |b| {
            b.iter(|| {
                for _ in 0..cycles {
                    for k in &keys {
                        black_box(hm.get(k));
                    }
                }
            })
        });
        group.bench_function("HashMap<String>/contains", |b| {
            b.iter(|| {
                for _ in 0..cycles {
                    for k in &keys {
                        black_box(hm.contains_key(k));
                    }
                }
            })
        });
        group.bench_function("HashMap<String>/insert", |b| {
            b.iter(|| {
                let mut hm: std::collections::HashMap<String, Vec<u8>> =
                    std::collections::HashMap::new();
                for k in &keys {
                    hm.insert(k.clone(), val.clone());
                }
            })
        });
    }

    group.finish();
}

fn kv_workload_10k(c: &mut Criterion) {
    kv_workload(c, "10k", 10_000, 1);
}
fn kv_workload_1m(c: &mut Criterion) {
    kv_workload(c, "1M", 65_536, 15);
}
fn kv_workload_10m(c: &mut Criterion) {
    kv_workload(c, "10M", 65_536, 153);
}

criterion_group!(
    name = kv;
    config = Criterion::default().sample_size(10);
    targets = bench_kv_single_insert_static, bench_kv_single_insert_dyn, bench_kv_single_insert_hashmap,
              bench_kv_single_get_static,    bench_kv_single_get_dyn,    bench_kv_single_get_hashmap,
              bench_kv_batch_insert_static_short, bench_kv_batch_insert_dyn_short, bench_kv_batch_insert_hashmap_short,
              bench_kv_batch_insert_static_medium, bench_kv_batch_insert_dyn_medium, bench_kv_batch_insert_hashmap_medium,
              bench_kv_batch_get_all,
              bench_kv_wrapper_single_insert, bench_kv_wrapper_single_get,
              bench_kv_wrapper_batch_get,
              kv_workload_10k, kv_workload_1m, kv_workload_10m,
);

criterion_main!(
    inserts,
    lookup,
    mutate,
    iterate,
    micro,
    n2_comparison,
    stress,
    spatial,
    n_scaling,
    large,
    edge,
    deep,
    set,
    kv
);

// ===========================================================================
// CoordSetN (N=2) benchmarks
// ===========================================================================

criterion_group!(
    set,
    bench_coordsetn_insert_1000_n2,
    bench_coordsetn_union_n2,
    bench_coordsetn_iter_1000_n2
);

// ===========================================================================
// CoordSpaceN6 (N=6, UUID-scale) benchmarks — 1,000 entries at depth 6
//
// Path distribution: 2 × 5 × 5 × 5 × 4 × 1 = 1,000 entries
// Memory: ~2 + 10 + 50 + 250 + 1000 + 1000 nodes → ~135MB total
//   All 1,000 entries share only 2 first-coord roots, 10 depth-2 branches.
//   At depth 5 each entry gets its own branch (the "sparse middle").
//   At depth 6 each entry gets its own leaf.
// ===========================================================================

// Edge/cs6_sparse_1k: 1,000 entries at depth 6, get + iter + axis projection.
fn bench_cs6_sparse_1k(c: &mut Criterion) {
    let mut cs6 = tagma_core::CoordSpaceN6::<u32>::new();
    let mut hm: std::collections::HashMap<[u16; 6], u32> = std::collections::HashMap::new();
    let mut paths = Vec::with_capacity(1000);
    let mk = |v: u16| tagma_core::Coord::new(v).unwrap();

    for a in 0u16..2 {
        for b in 0u16..5 {
            for cc in 0u16..5 {
                for d in 0u16..5 {
                    for e in 0u16..4 {
                        let path = tagma_core::CoordPath::new([
                            mk(a * 5586),
                            mk(1117 + b * 2234),
                            mk(2234 + cc * 1117),
                            mk(4468 + d * 1117),
                            mk(6698 + e * 1117),
                            mk(8930 + (a + b + cc + d + e) % 11172),
                        ]);
                        let val = a as u32 * 100_000
                            + b as u32 * 20_000
                            + cc as u32 * 4_000
                            + d as u32 * 800
                            + e as u32 * 200;
                        let hkey: [u16; 6] = std::array::from_fn(|i| path.coords()[i].index());
                        paths.push((path, hkey));
                        cs6.place_path(&path, val);
                        hm.insert(hkey, val);
                    }
                }
            }
        }
    }

    let mut group = c.benchmark_group("Edge/cs6_sparse_1k");
    group.throughput(criterion::Throughput::Elements(1000));

    group.bench_function("CoordSpaceN6/get", |b| {
        b.iter(|| {
            for (path, _) in &paths {
                black_box(cs6.at_path(path));
            }
        })
    });
    group.bench_function("HashMap/get", |b| {
        b.iter(|| {
            for (_, hk) in &paths {
                black_box(hm.get(hk));
            }
        })
    });
    group.bench_function("CoordSpaceN6/iter", |b| {
        b.iter(|| black_box(cs6.iter_tree().count()))
    });
    group.bench_function("HashMap/iter", |b| b.iter(|| black_box(hm.len())));

    group.finish();
}

// Edge/cs6_nonexistent_prefix: query a nonexistent prefix against 1,000 stored entries.
// CoordSpaceN6 navigates depth-5 branch, finds None → immediate return.
// HashMap scans all 1,000 entries, finds none → O(N).
fn bench_cs6_nonexistent_prefix(c: &mut Criterion) {
    let mut cs6 = tagma_core::CoordSpaceN6::<u32>::new();
    let mut hm: std::collections::HashMap<[u16; 6], u32> = std::collections::HashMap::new();
    let mk = |v: u16| tagma_core::Coord::new(v).unwrap();

    for a in 0u16..2 {
        for b in 0u16..5 {
            for cc in 0u16..5 {
                for d in 0u16..5 {
                    for e in 0u16..4 {
                        let path = tagma_core::CoordPath::new([
                            mk(a * 5586),
                            mk(1117 + b * 2234),
                            mk(2234 + cc * 1117),
                            mk(4468 + d * 1117),
                            mk(6698 + e * 1117),
                            mk(8930 + (a + b + cc + d + e) % 11172),
                        ]);
                        let val = a as u32 * 100_000
                            + b as u32 * 20_000
                            + cc as u32 * 4_000
                            + d as u32 * 800
                            + e as u32 * 200;
                        let hkey: [u16; 6] = std::array::from_fn(|i| path.coords()[i].index());
                        cs6.place_path(&path, val);
                        hm.insert(hkey, val);
                    }
                }
            }
        }
    }

    let missing_prefix = tagma_core::CoordPath::new([mk(9999), mk(0), mk(0), mk(0), mk(0), mk(0)]);

    let mut group = c.benchmark_group("Edge/cs6_nonexistent_prefix");
    group.bench_function("CoordSpaceN6", |b| {
        b.iter(|| black_box(cs6.at_path(&missing_prefix)))
    });
    group.bench_function("HashMap", |b| {
        let hk: [u16; 6] = std::array::from_fn(|i| missing_prefix.coords()[i].index());
        b.iter(|| black_box(hm.get(&hk)))
    });
    group.finish();
}

// Edge/cs6_md_axis_projection: multi-dimensional axis filter at depth 6.
// Query: "count entries where coord[2].initial == 2 AND coord[4].medial == 3"
// over 1,000 stored entries at CoordSpaceN6.
// Both sides decompose each Coord into axes and filter.
fn bench_cs6_md_axis_projection(c: &mut Criterion) {
    let mut cs6 = tagma_core::CoordSpaceN6::<u32>::new();
    let mut hm: std::collections::HashMap<[u16; 6], u32> = std::collections::HashMap::new();
    let mk = |v: u16| tagma_core::Coord::new(v).unwrap();

    for a in 0u16..2 {
        for b in 0u16..5 {
            for cc in 0u16..5 {
                for d in 0u16..5 {
                    for e in 0u16..4 {
                        let path = tagma_core::CoordPath::new([
                            mk(a * 5586),
                            mk(1117 + b * 2234),
                            mk(2234 + cc * 1117),
                            mk(4468 + d * 1117),
                            mk(6698 + e * 1117),
                            mk(8930 + (a + b + cc + d + e) % 11172),
                        ]);
                        let val = a as u32 * 100_000
                            + b as u32 * 20_000
                            + cc as u32 * 4_000
                            + d as u32 * 800
                            + e as u32 * 200;
                        let hkey: [u16; 6] = std::array::from_fn(|i| path.coords()[i].index());
                        cs6.place_path(&path, val);
                        hm.insert(hkey, val);
                    }
                }
            }
        }
    }

    let mut group = c.benchmark_group("Edge/cs6_md_axis_projection");
    group.throughput(criterion::Throughput::Elements(1000));

    // Filter: coord[2].initial == 2 AND coord[4].medial == 3
    group.bench_function("CoordSpaceN6", |b| {
        b.iter(|| {
            let count = cs6
                .iter_tree()
                .filter(|(path, _)| {
                    path.coords()[2].to_axes().0 == 2 && path.coords()[4].to_axes().1 == 3
                })
                .count();
            black_box(count);
        })
    });

    group.bench_function("HashMap", |b| {
        b.iter(|| {
            let count = hm
                .iter()
                .filter(|(hk, _)| {
                    tagma_core::Coord::new(hk[2]).unwrap().to_axes().0 == 2
                        && tagma_core::Coord::new(hk[4]).unwrap().to_axes().1 == 3
                })
                .count();
            black_box(count);
        })
    });

    group.finish();
}

// ===========================================================================
// CoordSpaceN19 (N=19, SHA-256-scale) benchmarks — 100 entries at depth 19
//
// Path distribution: 2 × 2 × 1 × ... × 25 = 100 entries
//   All entries share first 18 coords (2 unique at depth 1-2, 1 each at 3-18).
//   Only the 19th coord varies (25 values, 4 entries per value via 4 paths per leaf).
//   Memory: 2 + 4 + 16 × 1 + 1 + 25 nodes ≈ 48 nodes total → ~4.2MB
// ===========================================================================

// Edge/cs19_sparse_100: 100 entries at depth 19, get + iter.
fn bench_cs19_sparse_100(c: &mut Criterion) {
    let mut cs19 = tagma_core::CoordSpaceN19::<u32>::new();
    let mut hm: std::collections::HashMap<[u16; 19], u32> = std::collections::HashMap::new();
    let mut paths = Vec::with_capacity(100);
    let mk = |v: u16| tagma_core::Coord::new(v).unwrap();

    // 2 × 2 prefixes, then 25 unique 19th coords
    for a in 0u16..2 {
        for b in 0u16..2 {
            for z in 0u16..25 {
                // First 18 coords are mostly fixed, only 19th varies
                let mut coords = [mk(0u16); 19];
                coords[0] = mk(a * 5586);
                coords[1] = mk(1117 + b * 2234);
                coords[18] = mk(4468 + z * 279);

                let path = tagma_core::CoordPath::new(coords);
                let val = a as u32 * 50 + b as u32 * 25 + z as u32;
                let hkey: [u16; 19] = std::array::from_fn(|i| path.coords()[i].index());
                paths.push((path, hkey));
                cs19.place_path(&path, val);
                hm.insert(hkey, val);
            }
        }
    }

    let mut group = c.benchmark_group("Edge/cs19_sparse_100");
    group.throughput(criterion::Throughput::Elements(100));

    group.bench_function("CoordSpaceN19/get", |b| {
        b.iter(|| {
            for (path, _) in &paths {
                black_box(cs19.at_path(path));
            }
        })
    });
    group.bench_function("HashMap/get", |b| {
        b.iter(|| {
            for (_, hk) in &paths {
                black_box(hm.get(hk));
            }
        })
    });

    group.finish();
}

// Edge/cs19_nonexistent_prefix: query a nonexistent prefix against 100 stored entries at depth 19.
// CoordSpaceN19 navigates depth-18 branch, finds None → immediate.
// HashMap scans all 100 entries.
fn bench_cs19_nonexistent_prefix(c: &mut Criterion) {
    let mut cs19 = tagma_core::CoordSpaceN19::<u32>::new();
    let mut hm: std::collections::HashMap<[u16; 19], u32> = std::collections::HashMap::new();
    let mk = |v: u16| tagma_core::Coord::new(v).unwrap();

    for a in 0u16..2 {
        for b in 0u16..2 {
            for z in 0u16..25 {
                let mut coords = [mk(0u16); 19];
                coords[0] = mk(a * 5586);
                coords[1] = mk(1117 + b * 2234);
                coords[18] = mk(4468 + z * 279);
                let path = tagma_core::CoordPath::new(coords);
                let val = a as u32 * 50 + b as u32 * 25 + z as u32;
                let hkey: [u16; 19] = std::array::from_fn(|i| path.coords()[i].index());
                cs19.place_path(&path, val);
                hm.insert(hkey, val);
            }
        }
    }

    let mut missing_coords = [mk(0u16); 19];
    missing_coords[0] = mk(9999); // nonexistent first coord
    let missing_prefix = tagma_core::CoordPath::new(missing_coords);

    let mut group = c.benchmark_group("Edge/cs19_nonexistent_prefix");
    group.bench_function("CoordSpaceN19", |b| {
        b.iter(|| black_box(cs19.at_path(&missing_prefix)))
    });
    group.bench_function("HashMap", |b| {
        let hk: [u16; 19] = std::array::from_fn(|i| missing_prefix.coords()[i].index());
        b.iter(|| black_box(hm.get(&hk)))
    });
    group.finish();
}

// ===========================================================================
// CoordSetN benchmarks
// ===========================================================================

fn bench_coordsetn_insert_1000_n2(c: &mut Criterion) {
    use tagma_core::CoordSetN;

    let mk = |v: u16| tagma_core::Coord::new(v).unwrap();
    let mut paths: Vec<tagma_core::CoordPath<2>> = Vec::with_capacity(1000);
    let mut hkeys: Vec<[u16; 2]> = Vec::with_capacity(1000);
    for i in 0u16..1000 {
        let p = tagma_core::CoordPath::new([mk(i), mk(i + 100)]);
        hkeys.push([p.coords()[0].index(), p.coords()[1].index()]);
        paths.push(p);
    }

    let mut group = c.benchmark_group("CoordSetN<2>/insert/1000");
    group.bench_function("CoordSetN", |b| {
        b.iter(|| {
            let mut s = CoordSetN::<2>::new();
            for p in &paths {
                black_box(s.insert(*p));
            }
            black_box(s.len());
        })
    });
    group.bench_function("HashSet", |b| {
        b.iter(|| {
            let mut s = std::collections::HashSet::new();
            for k in &hkeys {
                black_box(s.insert(*k));
            }
            black_box(s.len());
        })
    });
    group.finish();
}

fn bench_coordsetn_union_n2(c: &mut Criterion) {
    use tagma_core::CoordSetN;

    let mk = |v: u16| tagma_core::Coord::new(v).unwrap();
    let mut a = CoordSetN::<2>::new();
    let mut b = CoordSetN::<2>::new();
    let mut ha = std::collections::HashSet::new();
    let mut hb = std::collections::HashSet::new();
    for i in 0u16..500 {
        let p = tagma_core::CoordPath::new([mk(i), mk(i)]);
        a.insert(p);
        ha.insert([mk(i).index(), mk(i).index()]);
    }
    for i in 250u16..750 {
        let p = tagma_core::CoordPath::new([mk(i), mk(i)]);
        b.insert(p);
        hb.insert([mk(i).index(), mk(i).index()]);
    }

    let mut group = c.benchmark_group("CoordSetN<2>/set_ops");
    group.bench_function("union_500_500", |bench| {
        bench.iter(|| black_box(a.union(&b)))
    });
    group.bench_function("intersection_500_500", |bench| {
        bench.iter(|| black_box(a.intersection(&b)))
    });
    group.bench_function("difference_500_500", |bench| {
        bench.iter(|| black_box(a.difference(&b)))
    });
    group.bench_function("is_subset_500_500", |bench| {
        bench.iter(|| black_box(a.is_subset(&b)))
    });
    group.bench_function("is_disjoint_500_500", |bench| {
        bench.iter(|| black_box(a.is_disjoint(&b)))
    });
    group.bench_function("HashSet_union_500_500", |bench| {
        bench.iter(|| black_box(&ha | &hb))
    });
    group.bench_function("HashSet_is_disjoint_500_500", |bench| {
        bench.iter(|| black_box(ha.is_disjoint(&hb)))
    });
    group.finish();
}

fn bench_coordsetn_iter_1000_n2(c: &mut Criterion) {
    use tagma_core::CoordSetN;

    let mut set = CoordSetN::<2>::new();
    let mk = |v: u16| tagma_core::Coord::new(v).unwrap();
    for i in 0u16..1000 {
        set.insert(tagma_core::CoordPath::new([mk(i), mk(i + 100)]));
    }

    c.bench_function("CoordSetN<2>/iter/1000", |b| {
        b.iter(|| {
            let mut count = 0usize;
            for (_, _) in set.iter() {
                count += 1;
            }
            black_box(count);
        })
    });
}
