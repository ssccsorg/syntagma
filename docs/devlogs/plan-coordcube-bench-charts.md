# CoordCube Benchmark Chart Plan

## Measured Data (ARMv8.4-A Firestorm)

Map-store rows (sections 8 and 9), the hierarchical rows, and the distance-metric rows were re-measured on 2026-09-08 after the CoordCubeMap byte-domain change (issue #59). Map-store queries operate in the per-character domain [0, 256) with byte-domain fills (center 136, bytes 86..=186) instead of coordinates around 5000 that wrapped through the key conversion.

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

### 6. Distance metrics (single pair, runtime inputs)

Measured with the runtime-generated `bench_coordcube_distance_metrics` benchmark, whose random input buffer defeats constant folding:

| Metric | R=1 | R=2 |
|--------|-----|-----|
| Hamming | 1.74 ns | 2.28 ns |
| Euclidean approx | 13.41 ns | 13.25 ns |
| Manhattan | 2.61 ns | 2.59 ns |


### 7. CoordSet compound axis query

| Implementation | Time | Throughput | vs HashMap |
|---------------|------|------------|------------|
| CoordSet (bitwise AND) | 85.7 ns | 327 Melem/s | **144x** |
| HashMap (iterate + filter) | 12.3 us | 2.28 Melem/s | baseline |

**Insight**: CoordSet pre-computed per-axis bit sets answer compound axis queries with a single bitwise AND (1.4 KB). HashMap must scan all 11,172 entries every time.

### 8. CoordCube vs CoordPath on the map store (KEY COMPARISON)

Direct comparison on the same 10K-entry CoordMapN<2> store (byte-domain fill, bytes 86..=186):

| Method | Time | Paths generated | Per-path cost | Overhead vs baseline |
|--------|------|----------------|---------------|---------------------|
| Sequential path lookup (baseline) | 160.5 ns | 9 (manual) | 17.8 ns/lookup | -- |
| CoordCube proximity r=1 | 240.7 ns | 9 | 26.7 ns/lookup | +80.2 ns (~50%) |
| CoordCube proximity r=2 | 556.2 ns | 25 | 22.2 ns/lookup | +396 ns (~247%) |
| DynCoordMap proximity r=1 | 175.6 ns | 9 | 19.5 ns/lookup | +15 ns (~9%) |
| DynCoordMap proximity r=2 | 386.4 ns | 25 | 15.5 ns/lookup | +226 ns (~141%) |

**Breakdown of the ~80 ns overhead in Tree+Cube r=1**: generation alone is about 12 ns for the 9-path box (cube_proximity_r1_baseline measures 12.3 ns), and the remaining ~68 ns is Vec push and allocation for the 9 hits. The earlier 127 ns split predates the byte-domain change and is superseded by this re-measurement.

**Insight**: CoordCube proximity overhead is dominated by Vec push and allocation, not by coordinate arithmetic; on short-lived queries it adds about 50% over sequential lookups at r=1. Empty and sparse stores finish faster because fewer lookups hit and fewer entries are pushed; the generation pass is constant for a given radius. The CoordCubeMap path performs no structural short-circuit: every generated path is looked up.

### 9. Map proximity at scale

| Scenario | Store | Query | Time | Found |
|----------|-------|-------|------|-------|
| Dense 10K entries (bytes 86..=186) | CoordMapN<2> | r=1 | 237.9 ns | 9 |
| Dense 10K entries (bytes 86..=186) | CoordMapN<2> | r=2 | 530.2 ns | 25 |
| Dense 2.6K entries (bytes 111..=161) | CoordMapN<2> | r=5 | 2.29 us | 121 |
| Sparse 9 entries (bytes {86, 136, 186}) | CoordMapN<2> | r=1 | 84.4 ns | 1 |
| Empty store | CoordMapN<2> | r=1 | 63.9 ns | 0 |
| DynCoordMap 100 entries | DynCoordMap | r=1 | 175.6 ns | -- |
| DynCoordMap 100 entries | DynCoordMap | r=2 | 386.4 ns | -- |
| Hierarchical R=2, r=1 (2-phase) | CoordMapN<4> | r=1 | 550.5 ns | -- |
| Hierarchical R=2, r=1 (direct map) | CoordMapN<4> | r=1 | 322.1 ns | -- |

**Hierarchical insight**: Direct map proximity (322 ns) beats the manual two-phase filter (550 ns) on the R=2 store after the bounded-generation change (issue #59). The direct path generates candidates bounded to the store domain and pushes only hits in a single pass; the manual phase collects all 3^4 candidates into a Vec first and re-looks each one up. The earlier record showed the reverse order and predated the change. The sparse row records 1 hit, not 9: the nine scattered bytes {86, 136, 186} place only the center entry inside the radius-1 box around byte 136.

### 10. Large N

| Configuration | Time | Notes |
|---------------|------|-------|
| N=12 path gen r=0 | 21.1 ns | 12D proximity with radius 0 (single path) |
| N=12 map proximity r=0 | 127.3 ns | path gen + tree lookup |
| N=19 path gen r=0 | 36.4 ns | 19D proximity with radius 0 (single path) |

## Proposed Charts

### Chart 1: CoordCube Overhead -- Path vs Cube (bar)

File: fig-bench-coordcube-overhead.qmd

Two groups of bars:
1. raw_path_get_3x (318 ps) vs cube_axis_3x (322 ps) -- showing zero cost
2. cube_from_path (961 ps) -- showing construction cost

Annotate: "Zero-cost interpretation layer" / "0.96 ns one-time cost"

### Chart 2: Proximity Throughput Scaling (line + scatter)

File: fig-bench-coordcube-proximity.qmd

X-axis: radius (0-5)
Y-axis: Throughput (Melem/s)
Line shows throughput increasing ~54 to 355 Melem/s as radius grows
Second line for bounding box at N=6 showing consistent ~430 Melem/s

### Chart 3: Dimensional vs Resolution Scaling (grouped bar)

File: fig-bench-coordcube-dim-res.qmd

Two groups showing identical pattern:
- D=1 to D=4 (proximity r=2)
- R=1 to R=3 (proximity r=2)
Highlight that D*R = N is the real driver, not D or R individually

### Chart 4: Tree+Path vs Tree+Cube -- The Cost of Convenience (bar)

File: fig-bench-coordcube-vs-path.qmd

THE KEY CHART. Three bars:
- sequential 9 lookups: 160 ns
- CoordCube prox r=1: 241 ns (+50% overhead)
- CoordCube prox r=2: 556 ns

Breakdown callout showing the r=1 gap of ~80 ns: Vec push/alloc dominated, generation ~12 ns

### Chart 5: CoordCube Proximity -- Empty vs Sparse vs Dense (bar)

File: fig-bench-coordcube-density.qmd

Three bars at r=1:
- Empty store: 64 ns (path gen only, no lookups hit)
- Sparse 9 entries: 84 ns (1 hit)
- Dense 10K: 238 ns (all 9 lookups hit + Vec push)

Annotate: "Generation pass is constant; hits and Vec pushes vary"

### Chart 6: CoordSet Compound Axis Query (bar)

File: fig-bench-coordcube-coordset.qmd

Two bars: CoordSet 84.2 ns vs HashMap 13.0 us
Annotate: "~154x faster: single bitwise AND vs 11,172 iterations"

### Chart 7: Large-N Scaling (bar)

File: fig-bench-coordcube-largen.qmd

N=12 path gen r=0: 21 ns
N=12 map prox r=0: 127 ns
N=6 bbox 3^6: 1.70 us

## Distance Metrics -- RESOLVED

The distance metrics benchmark now generates runtime random inputs (`bench_coordcube_distance_metrics`), so the measured values in section 6 are representative and chartable.

## Implementation Order

1. fig-bench-coordcube-overhead.qmd -- simplest, establishes zero-cost
2. fig-bench-coordcube-vs-path.qmd -- key insight chart
3. fig-bench-coordcube-proximity.qmd -- throughput scaling
4. fig-bench-coordcube-density.qmd -- store density effects
5. fig-bench-coordcube-dim-res.qmd -- dimension vs resolution
6. fig-bench-coordcube-coordset.qmd -- compound axis
7. fig-bench-coordcube-largen.qmd -- large-N capability
