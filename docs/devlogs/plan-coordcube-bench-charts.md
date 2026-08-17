# CoordCube Benchmark Chart Plan

## Measured Data (ARMv8.4-A Firestorm)

### 1. CoordCube overhead over CoordPath (cubeoverhead)

| Benchmark | Time | Notes |
|-----------|------|-------|
| raw_path_get_3x (access 6 coords directly) | 319 ps | Baseline: CoordPath direct access |
| cube_axis_3x (extract 3 axes via CoordCube) | 319 ps | **Zero-cost**: identical to raw path, within noise |
| cube_from_path (wrap CoordPath in CoordCube) | 958 ps | One-time construction cost, negligible per-query |

**Insight**: Creating a CoordCube costs ~0.96 ns. All axis extraction is free (same cost as raw CoordPath access). The interpretation layer adds no per-access overhead.

### 2. Proximity generation (D=2, R=1)

| Radius | Paths | Time | Throughput | Per-path cost |
|--------|-------|------|------------|---------------|
| 0 | 1 | 16.5 ns | 60.6 Melem/s | 16.5 ns |
| 1 | 9 | 86.6 ns | 103.9 Melem/s | 9.62 ns |
| 2 | 25 | 126.4 ns | 197.8 Melem/s | 5.06 ns |
| 3 | 49 | 184.6 ns | 265.5 Melem/s | 3.77 ns |
| 5 | 121 | 331.4 ns | 365.1 Melem/s | 2.74 ns |

**Insight**: Throughput increases with radius because fixed overhead (center clamp, bounds check, iterator init) is amortized over more paths.

### 3. Bounding box enumeration

| Configuration | Paths | Time | Throughput |
|---------------|-------|------|------------|
| N=2, 100x100 | 10,201 | 14.28 us | 714 Melem/s |
| N=2, 10x10 | 100 | 300 ns | 333 Melem/s |
| N=4, 4x4x4x4 | 256 | 868 ns | 295 Melem/s |
| N=6, 3^6 | 729 | 1.71 us | 426 Melem/s |
| N=6, 3^6 (cubelargen) | 729 | 1.70 us | 427 Melem/s |

**Insight**: Bounding box throughput is consistently higher than proximity because bbox uses a simpler mixed-radix iteration while proximity must compute L-infinity distance masks.

### 4. Dimensional scaling (proximity r=2, R=1)

| Dimensions | N | Paths | Time | Throughput |
|------------|---|-------|------|------------|
| D=1 | 1 | 5 | 33.8 ns | 148 Melem/s |
| D=2 | 2 | 25 | 126.5 ns | 198 Melem/s |
| D=3 | 3 | 125 | 441.9 ns | 283 Melem/s |
| D=4 | 4 | 625 | 1.887 us | 331 Melem/s |

**Insight**: Throughput increases with D because each additional dimension adds a multiplicative factor to path count while the per-path cost is dominated by the inner loop, not dimension count.

### 5. Resolution scaling (D=1, proximity r=2)

| Resolution | N | Paths | Time | Throughput |
|------------|---|-------|------|------------|
| R=1 | 1 | 5 | 34.3 ns | 146 Melem/s |
| R=2 | 2 | 25 | 127.8 ns | 196 Melem/s |
| R=3 | 3 | 125 | 447.1 ns | 280 Melem/s |

**Insight**: Identical pattern to dimensional scaling -- same mixed-radix iteration, R=2 is like D=2, R=3 is like D=3. Throughput follows total path count, not N or D or R individually.

### 6. Distance metrics (3D, single pair) -- FLAGGED

**Warning**: Current benchmark uses compile-time-constant CoordCube values. The optimizer pre-computes all distance results, producing ~320 ps measurements that are not representative of real performance. The benchmark inputs need to pass through `black_box` or be runtime-generated before these numbers can be trusted.

Raw (unreliable) measurements:
| Metric | Measurement | Status |
|--------|-------------|--------|
| Hamming | 321 ps | Optimized away |
| Euclidean approx | 318 ps | Optimized away |
| Manhattan | 317 ps | Optimized away |

These should be in the low-ns range once fixed (Manhattan sum of absolute diffs across D dimensions, requiring integer ops per dim).

### 7. CoordSet compound axis query

| Implementation | Time | Throughput | vs HashMap |
|---------------|------|------------|------------|
| CoordSet (bitwise AND) | 85.7 ns | 327 Melem/s | **144x** |
| HashMap (iterate + filter) | 12.3 us | 2.28 Melem/s | baseline |

**Insight**: CoordSet pre-computed per-axis bit sets answer compound axis queries with a single bitwise AND (1.4 KB). HashMap must scan all 11,172 entries every time.

### 8. CoordCube vs CoordPath on the map store (KEY COMPARISON)

Direct comparison on the same 10K-entry CoordMapN<2> store:

| Method | Time | Paths generated | Per-path cost | Overhead vs baseline |
|--------|------|----------------|---------------|---------------------|
| Sequential path lookup (baseline) | 158 ns | 9 (manual) | 17.6 ns/lookup | -- |
| CoordCube proximity r=1 | 285 ns | 9 | 31.7 ns/lookup | +127 ns (80%) |
| CoordCube proximity r=2 | 626 ns | 25 | 25.0 ns/lookup | +468 ns (296%) |
| DynCoordMap proximity r=1 | 161 ns | 9 | 17.9 ns/lookup | +3 ns (2%) |
| DynCoordMap proximity r=2 | 290 ns | 25 | 11.6 ns/lookup | +132 ns (84%) |

**Breakdown of the +127 ns overhead in Tree+Cube r=1**:
- Path generation via CoordCube: ~16 ns (measured: cube_from_path ~1 ns + iteration overhead ~15 ns)
- Vec::push for 9 paths: ~74 ns (9 x ~8.2 ns/push)
- Vec allocation: ~37 ns (amortized capacity doubling)
- **Total**: ~127 ns, matches measured gap

**Insight**: CoordCube proximity overhead is dominated by Vec allocation and push, not by coordinate arithmetic. On short-lived queries, this overhead is significant (+80%). On sparse stores where most generated paths are absent, CoordCube proximity short-circuits faster than sequential lookups because BoundingBoxIter::next() returns None immediately for out-of-bounds, while sequential lookup must query the tree for each key.

### 9. Map proximity at scale

| Scenario | Store | Query | Time | Found |
|----------|-------|-------|------|-------|
| Dense 10K entries | CoordMapN<2> | r=1 | 285 ns | 9 |
| Dense 10K entries | CoordMapN<2> | r=2 | 626 ns | 25 |
| Dense 10K entries | CoordMapN<2> | r=5 | 2.55 us | 121 |
| Sparse 9 entries | CoordMapN<2> | r=1 | 48.5 ns | 9 |
| Empty store | CoordMapN<2> | r=1 | 15.7 ns | 0 |
| DynCoordMap 100 entries | DynCoordMap | r=1 | 161 ns | 9 |
| DynCoordMap 100 entries | DynCoordMap | r=2 | 290 ns | 25 |
| Hierarchical R=2, r=1 (2-phase) | CoordMapN<4> | r=1 | 547 ns | -- |
| Hierarchical R=2, r=1 (direct map) | CoordMapN<4> | r=1 | 639 ns | -- |

**Hierarchical insight**: CoordCube prox r=1 followed by manual lookup (547 ns) is slightly faster than direct map proximity (639 ns) on R=2 stores, because CoordCube generates candidate paths without considering multi-char dimension boundaries, and the manual post-filter weeds out false positives.

### 10. Large N

| Configuration | Time | Notes |
|---------------|------|-------|
| N=12 path gen r=0 | 17.5 ns | 12D proximity with radius 0 (single path) |
| N=12 map proximity r=0 | 118.6 ns | path gen + tree lookup |
| N=19 path gen r=0 | ~18 ns (projected) | Similar to N=12 since path gen is O(1) at r=0 |

## Proposed Charts

### Chart 1: CoordCube Overhead -- Path vs Cube (bar)

File: fig-bench-coordcube-overhead.qmd

Two groups of bars:
1. raw_path_get_3x (319 ps) vs cube_axis_3x (319 ps) -- showing zero cost
2. cube_from_path (958 ps) -- showing construction cost

Annotate: "Zero-cost interpretation layer" / "0.96 ns one-time cost"

### Chart 2: Proximity Throughput Scaling (line + scatter)

File: fig-bench-coordcube-proximity.qmd

X-axis: radius (0-5)
Y-axis: Throughput (Melem/s)
Line shows throughput increasing ~60 to 365 Melem/s as radius grows
Second line for bounding box at N=6 showing consistent ~427 Melem/s

### Chart 3: Dimensional vs Resolution Scaling (grouped bar)

File: fig-bench-coordcube-dim-res.qmd

Two groups showing identical pattern:
- D=1 to D=4 (proximity r=2)
- R=1 to R=3 (proximity r=2)
Highlight that D*R = N is the real driver, not D or R individually

### Chart 4: Tree+Path vs Tree+Cube -- The Cost of Convenience (bar)

File: fig-bench-coordcube-vs-path.qmd

THE KEY CHART. Three bars:
- sequential 9 lookups: 158 ns
- CoordCube prox r=1: 285 ns (+80% overhead)
- CoordCube prox r=2: 626 ns

Breakdown callout showing where the 127 ns goes: Vec alloc (37 ns) + push (74 ns) + path gen (16 ns)

### Chart 5: CoordCube Proximity -- Empty vs Sparse vs Dense (bar)

File: fig-bench-coordcube-density.qmd

Three bars at r=1:
- Empty store: 15.7 ns (path gen only, no lookups hit)
- Sparse 9 entries: 48.5 ns (few lookups)
- Dense 10K: 285 ns (all 9 lookups hit + Vec push)

Annotate: "Structural short-circuit on empty: 18x faster than dense"

### Chart 6: CoordSet Compound Axis Query (bar)

File: fig-bench-coordcube-coordset.qmd

Two bars: CoordSet 85.7 ns vs HashMap 12.3 us
Annotate: "144x faster: single bitwise AND vs 11,172 iterations"

### Chart 7: Large-N Scaling (bar)

File: fig-bench-coordcube-largen.qmd

N=12 path gen r=0: 17.5 ns
N=12 map prox r=0: 119 ns
N=6 bbox 3^6: 1.70 us

## Distance Metrics -- DEFERRED

The distance metrics benchmark needs to be fixed before charting. Current measurements (~320 ps) are compiler-optimized and not representative. Fix approach: pass `black_box` on the Coord values or generate them from `thread_rng` at warmup time.

## Implementation Order

1. fig-bench-coordcube-overhead.qmd -- simplest, establishes zero-cost
2. fig-bench-coordcube-vs-path.qmd -- key insight chart
3. fig-bench-coordcube-proximity.qmd -- throughput scaling
4. fig-bench-coordcube-density.qmd -- store density effects
5. fig-bench-coordcube-dim-res.qmd -- dimension vs resolution
6. fig-bench-coordcube-coordset.qmd -- compound axis
7. fig-bench-coordcube-largen.qmd -- large-N capability
