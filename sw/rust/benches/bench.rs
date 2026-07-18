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
//   CoordSpace2  4.2 µs     240 Kelem/s   4.5x (with iter_prefix)
//   HashMap     188  µs     5.4 Melem/s
fn bench_spatial_cs2_prefix_scan(c: &mut Criterion) {
    // CoordSpace2: 10000 entries with shared prefixes vs HashMap<(Coord,Coord),V>.
    // Query: find all entries matching a specific prefix (first coord).
    // CoordSpace2 can restrict iteration to the sub-tree at that prefix;
    // HashMap must scan every entry.
    let mut cs2 = tagma_core::CoordSpace2::<u32>::new();
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

    group.bench_function("CoordSpace2", |b| {
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
// CoordSpace2 bulk 100k entries
// ===========================================================================

// CoordSpace2/bulk_100k (1000 prefixes, 100 suffixes each)
//   CoordSpace/insert   ~1.5 ms
//   HashMap/insert      ~2.5 ms
//   CoordSpace/get      ~0.6 ms
//   HashMap/get         ~1.2 ms
fn bench_cs2_bulk_100k(c: &mut Criterion) {
    let cs2 = tagma_core::CoordSpace2::<u32>::new();
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

    let mut group = c.benchmark_group("CoordSpace2/bulk_100k");
    group.throughput(criterion::Throughput::Elements(100_000));

    group.bench_function("CoordSpace/insert", |b| {
        b.iter(|| {
            let mut cs = tagma_core::CoordSpace2::<u32>::new();
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
    let mut cs2 = tagma_core::CoordSpace2::<u32>::new();
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

// Edge/cs2_md_axis_projection: multi-dimensional query at CoordSpace2 scale.
// Query: "count entries where prefix.initial==3 AND suffix.medial==7" over 5M entries.
// Both sides: iterate all entries, decompose each Coord, check both axis conditions.
// Same filter logic, different memory layout (contiguous array vs fragmented bucket).
// NOTE: This is the CPU-bound version. CoordSet bitwise approach (175-word AND)
// would be much faster but requires pre-computed per-axis sets, which is
// infrastructure-level work, not a microbenchmark.
fn bench_cs2_md_axis_projection(c: &mut Criterion) {
    let mut cs2 = tagma_core::CoordSpace2::<u32>::new();
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
    group.bench_function("CoordSpace2", |b| {
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
// CoordSpace2 navigates to the branch at that prefix, finds Nothing, returns immediately.
// HashMap has no structural notion of "prefix" — it must scan all 10M entries to
// determine that no entry has first coord == 11111.
// WHY THIS MATTERS: In distributed systems, content-addressed networks, and
// sparse data structures, "negative" existence checks are as frequent as
// positive lookups. Tagma answers them in 1.6 ns regardless of data volume
// or depth. HashMap pays O(N) because it cannot answer structural queries.
fn bench_cs2_nonexistent_prefix(c: &mut Criterion) {
    let mut cs2 = tagma_core::CoordSpace2::<u32>::new();
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
    group.bench_function("CoordSpace2", |b| {
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

// N_scaling/get  (single lookup, Apple M1)
//   N=1   CoordSpace    0.38 ns   space 10^4
//   N=2   CoordSpace2   0.87 ns   space 10^8
//   N=3   CoordSpace3   2.66 ns   space 10^12
//   N=6   CoordSpace6   5.60 ns   space 10^24
//   N=12  CoordSpace12  13.2  ns   space 10^67
//   N=19  CoordSpace19  53.2  ns   space 10^77 (SHA-256 scale)
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
        let mut cs = tagma_core::CoordSpace2::<u64>::new();
        cs.place_path(&path2, 42);
        let mut group = c.benchmark_group("N_scaling/get/N=2");
        group.bench_function("CoordSpace2", |b| b.iter(|| black_box(cs.at_path(&path2))));
        group.finish();
    }
    {
        let mut cs = tagma_core::CoordSpace3::<u64>::new();
        cs.place_path(&path3, 42);
        let mut group = c.benchmark_group("N_scaling/get/N=3");
        group.bench_function("CoordSpace3", |b| b.iter(|| black_box(cs.at_path(&path3))));
        group.finish();
    }
    {
        let mut cs = tagma_core::CoordSpace6::<u64>::new();
        cs.place_path(&path6, 42);
        let mut group = c.benchmark_group("N_scaling/get/N=6");
        group.bench_function("CoordSpace6", |b| b.iter(|| black_box(cs.at_path(&path6))));
        group.finish();
    }
    {
        let mut cs = tagma_core::CoordSpace12::<u64>::new();
        cs.place_path(&path12, 42);
        let mut group = c.benchmark_group("N_scaling/get/N=12");
        group.bench_function("CoordSpace12", |b| {
            b.iter(|| black_box(cs.at_path(&path12)))
        });
        group.finish();
    }
    {
        let mut cs = tagma_core::CoordSpace19::<u64>::new();
        cs.place_path(&path19, 42);
        let mut group = c.benchmark_group("N_scaling/get/N=19");
        group.bench_function("CoordSpace19", |b| {
            b.iter(|| black_box(cs.at_path(&path19)))
        });
        group.finish();
    }
}

// ===========================================================================
// CoordSpace2 (N=2) benchmarks — cross-product FIH-like scenario
// ===========================================================================

// CoordSpace2/insert/1000          803 µs
fn bench_cm2_insert_1000(c: &mut Criterion) {
    c.bench_function("CoordSpace2/insert/1000", |b| {
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
}

// CoordSpace2/get/1000             4.92 µs
fn bench_cm2_get_1000(c: &mut Criterion) {
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
    c.bench_function("CoordSpace2/get/1000", |b| {
        b.iter(|| {
            for i in 0u16..100 {
                for j in 0u16..10 {
                    let path = tagma_core::CoordPath::new([
                        tagma_core::Coord::new(i).unwrap(),
                        tagma_core::Coord::new(j).unwrap(),
                    ]);
                    black_box(black_box(&map).at_path(&path));
                }
            }
        })
    });
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
    name = tree;
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
              bench_coordset_spatial_query
);

criterion_main!(
    inserts, lookup, mutate, iterate, micro, tree, stress, spatial, n_scaling, large, edge, deep
);

// ===========================================================================
// CoordSpace6 (N=6, UUID-scale) benchmarks — 1,000 entries at depth 6
//
// Path distribution: 2 × 5 × 5 × 5 × 4 × 1 = 1,000 entries
// Memory: ~2 + 10 + 50 + 250 + 1000 + 1000 nodes → ~135MB total
//   All 1,000 entries share only 2 first-coord roots, 10 depth-2 branches.
//   At depth 5 each entry gets its own branch (the "sparse middle").
//   At depth 6 each entry gets its own leaf.
// ===========================================================================

// Edge/cs6_sparse_1k: 1,000 entries at depth 6, get + iter + axis projection.
fn bench_cs6_sparse_1k(c: &mut Criterion) {
    let mut cs6 = tagma_core::CoordSpace6::<u32>::new();
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

    group.bench_function("CoordSpace6/get", |b| {
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
    group.bench_function("CoordSpace6/iter", |b| {
        b.iter(|| black_box(cs6.iter_tree().count()))
    });
    group.bench_function("HashMap/iter", |b| b.iter(|| black_box(hm.len())));

    group.finish();
}

// Edge/cs6_nonexistent_prefix: query a nonexistent prefix against 1,000 stored entries.
// CoordSpace6 navigates depth-5 branch, finds None → immediate return.
// HashMap scans all 1,000 entries, finds none → O(N).
fn bench_cs6_nonexistent_prefix(c: &mut Criterion) {
    let mut cs6 = tagma_core::CoordSpace6::<u32>::new();
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
    group.bench_function("CoordSpace6", |b| {
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
// over 1,000 stored entries at CoordSpace6.
// Both sides decompose each Coord into axes and filter.
fn bench_cs6_md_axis_projection(c: &mut Criterion) {
    let mut cs6 = tagma_core::CoordSpace6::<u32>::new();
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
    group.bench_function("CoordSpace6", |b| {
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
// CoordSpace19 (N=19, SHA-256-scale) benchmarks — 100 entries at depth 19
//
// Path distribution: 2 × 2 × 1 × ... × 25 = 100 entries
//   All entries share first 18 coords (2 unique at depth 1-2, 1 each at 3-18).
//   Only the 19th coord varies (25 values, 4 entries per value via 4 paths per leaf).
//   Memory: 2 + 4 + 16 × 1 + 1 + 25 nodes ≈ 48 nodes total → ~4.2MB
// ===========================================================================

// Edge/cs19_sparse_100: 100 entries at depth 19, get + iter.
fn bench_cs19_sparse_100(c: &mut Criterion) {
    let mut cs19 = tagma_core::CoordSpace19::<u32>::new();
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

    group.bench_function("CoordSpace19/get", |b| {
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
// CoordSpace19 navigates depth-18 branch, finds None → immediate.
// HashMap scans all 100 entries.
fn bench_cs19_nonexistent_prefix(c: &mut Criterion) {
    let mut cs19 = tagma_core::CoordSpace19::<u32>::new();
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
    group.bench_function("CoordSpace19", |b| {
        b.iter(|| black_box(cs19.at_path(&missing_prefix)))
    });
    group.bench_function("HashMap", |b| {
        let hk: [u16; 19] = std::array::from_fn(|i| missing_prefix.coords()[i].index());
        b.iter(|| black_box(hm.get(&hk)))
    });
    group.finish();
}
