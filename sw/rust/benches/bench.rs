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

// Spatial/cs2_prefix_42
//   CoordSpace2  4.17 µs   240 Kelem/s   4.5x (with iter_prefix)
//   HashMap     18.8  µs   5.4 Melem/s
fn bench_spatial_cs2_prefix_scan(c: &mut Criterion) {
    // CoordSpace2: 10000 entries with shared prefixes vs HashMap<(Coord,Coord),V>.
    // Query: find all entries matching a specific prefix (first coord).
    // CoordSpace2 can restrict iteration to the sub-tree at that prefix;
    // HashMap must scan every entry.
    let mut cs2 = tagma_core::CoordSpace2::<u32>::new();
    let mut hm: std::collections::HashMap<(u16, u16), u32> =
        std::collections::HashMap::new();
    // 100 prefixes × 100 suffixes = 10,000 entries
    for p in 0u16..100 {
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
    let mut hm: std::collections::HashMap<tagma_core::Coord, ()> =
        std::collections::HashMap::new();
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
        group.bench_function("CoordSpace2", |b| {
            b.iter(|| black_box(cs.at_path(&path2)))
        });
        group.finish();
    }
    {
        let mut cs = tagma_core::CoordSpace3::<u64>::new();
        cs.place_path(&path3, 42);
        let mut group = c.benchmark_group("N_scaling/get/N=3");
        group.bench_function("CoordSpace3", |b| {
            b.iter(|| black_box(cs.at_path(&path3)))
        });
        group.finish();
    }
    {
        let mut cs = tagma_core::CoordSpace6::<u64>::new();
        cs.place_path(&path6, 42);
        let mut group = c.benchmark_group("N_scaling/get/N=6");
        group.bench_function("CoordSpace6", |b| {
            b.iter(|| black_box(cs.at_path(&path6)))
        });
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

criterion_main!(inserts, lookup, mutate, iterate, micro, tree, stress, spatial, n_scaling);
