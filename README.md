# synTagma

synTagma is a spatial coordinate space computing system built on Tagma, a 16-bit coordinate primitive embedded in the Unicode closed-form composition block (U+AC00--U+D7AF). Every valid 16-bit value decomposes into three independent axes, serving simultaneously as a 1-D address and a 3-D coordinate. The reference implementation is a `#![no_std]` Rust library.

Tagma is a primitive where the address is the coordinate -- not a flat pointer, but a point in an N-dimensional geometric space. This is made possible by a 16-bit Unicode block allocated to a 3-axis writing system, which provides a collision-free, hash-less, structurally addressable coordinate space.

## Layers

```
synTagma (system)
  └─ Coordination layer (protocol, topology, distributed resolver)
  └─ tagma-sec (security primitives: authority, integrity, audit, channel)
  └─ tagma-map (hashless map + CoordCubeMap spatial queries)
  └─ tagma-geo (CoordCube interpretation layer, spatial ops, distance metrics)
  └─ Tagma core primitive (Coord, CoordPath, CoordSet, CoordSetN, CoordCube, CoordSpace)
```

- Tagma -- the core primitive: a 16-bit structural coordinate with closed-form composition, zero collisions, and single-cycle combinational decoding, verified exhaustively in hardware against the Rust reference (`hw/README.md`). The atomic identity primitive.
- tagma-geo -- spatial operations built on CoordCube: proximity (L∞ Chebyshev radius), bounding box enumeration, Hamming distance, Euclidean distance (approximate), Manhattan distance. Depends only on tagma-core.
- tagma-map -- native CoordSpace map: accepts `&str` keys at HashMap-competitive speed, stores entries in Tagma coordinate space, exposes standard `insert`/`get`/`remove` API plus `CoordKey`-based access. Integrates `tagma-geo` via `CoordCubeMap` for zero-cost spatial queries on map data. Zero extra cost for spatial indexing.
- tagma-sec -- security primitives for coordination traffic: authority (CoordPath Exact/Prefix scope authorization), integrity (epoch-bound seals), audit (chained evidence log with inclusion proofs), channel (non-repudiation receipts). Depends on tagma-core. Defined in `docs/spec/tagma-sec.md`.
- synTagma coordination layer -- recursive coordinate space expansion, physical topology mapping, distributed routing, and consistency protocol. Defined in the [synTagma](https://docs.ssccs.org/projects/syntagma).

## Tagma primitive: Feature levels

Tagma provides a single feature gate: `alloc` (default: on). Without it (`--no-default-features`), all Tagma types are `no_std` + `no_alloc`.

| Level | Feature flags | Heap | Available types |
|-------|---------------|------|-----------------|
| no_alloc | (none) | Never | Coord, CoordPath, CoordSet, CoordCube, CoordSpace |
| alloc | `alloc` (default) | Optional | + CoordSetN, CoordSpaceN\<N\>, CoordSpace2, DynCoordSpace |
| mmap | `mmap` | Optional | + CoordSpaceM\<N\> |

## Tagma type reference

### Always available (no_std, no allocator)

| Type | Description | File |
|------|-------------|------|
| Coord | 16-bit atomic coordinate (0..11172), 3-axis composition/decomposition, Hamming distance, U+AC00--U+D7AF block display | `core/src/coord.rs` |
| CoordPath\<N\> | Index path (not a hash key), compile-time N-element Coord array | `core/src/coord_path.rs` |
| CoordSet | Bit array over 11,172 slots (1.4 KB). Union, intersection, difference, subset tests, `Copy` | `core/src/coord_set.rs` |
| CoordCube\<N, D, R\> | Interpretation layer over CoordPath: N chars as D-dimensional grid with R chars/dimension. L∞ proximity, bounding box, axis decomposition. `N == D * R` enforced at runtime. Zero-cost conversion to/from CoordPath. No heap. | `core/src/coord_cube.rs` |
| CoordSpace\<V\> | Single-character direct-address table. Inline `[Option<V>; 11172]` -- zero heap. O(1), no hashing, no collisions | `core/src/coord_space.rs` |
| base11172 | Self-validating serialization: arbitrary bytes to composition-block strings | `base11172/src/lib.rs` |

Test coverage: 360+ unit/integration tests + 26 doc-tests, all passing. Zero clippy warnings. CI runs `cargo fmt --check`, `cargo clippy`, `cargo build --release`, `cargo test --release`, `cargo build --no-default-features` (no_alloc verification).

### Requires alloc (default feature)

| Type | Description | File |
|------|-------------|------|
| CoordSpaceN\<N, V\> | N-level direct-address tree. Lazy heap allocation per node. `N` dereferences per lookup | `core/src/coord_space_n.rs` |
| CoordSpaceN2\<V\> | 2-character ($1.25 \times 10^8$ space). Type alias for `CoordSpaceN<2, V>` | `core/src/coord_space_n.rs` |
| CoordSpaceN3\<V\> | 3-character ($1.39 \times 10^{12}$ space). Type alias for `CoordSpaceN<3, V>` | `core/src/coord_space_n.rs` |
| CoordSpaceN6\<V\> | 6-character UUID-scale ($1.94 \times 10^{24}$). Type alias for `CoordSpaceN<6, V>` | `core/src/coord_space_n.rs` |
| CoordSpaceN12\<V\> | 12-character ($2.41 \times 10^{67}$). Type alias for `CoordSpaceN<12, V>` | `core/src/coord_space_n.rs` |
| CoordSpaceN19\<V\> | 19-character ($\approx 2^{256}$, SHA-256-scale). Type alias for `CoordSpaceN<19, V>` | `core/src/coord_space_n.rs` |
| CoordSpace2\<V\> | N=2 dense heap, 124M slots, single `alloc_zeroed`, true Tagma identity | `core/src/coord_space_dense.rs` |
| CoordSpaceM\<N, V\> | N≥3 mmap-backed dense (feature: `mmap`). Virtual address reservation with `MAP_NORESERVE` | `core/src/coord_space_m.rs` |
| CoordSpaceM3\<V\> | N=3 mmap dense. Type alias for `CoordSpaceM<3, V>` | `core/src/coord_space_m.rs` |
| CoordSetN\<N\> | Sparse N-dimensional set over CoordPath\<N\>. Union, intersection, difference, subset, disjoint. Heap-backed bit tree. | `core/src/coord_set_n.rs` |
| DynCoordSpace\<V\> | Variable-depth trie, `&[Coord]` runtime path. Mixed-depth slot (Both) preserves shallow values | `core/src/dyn_coord_space.rs` |

### tagma-geo: Spatial operations (requires alloc)

| Type | Description | File |
|------|-------------|------|
| SpatialOps trait | `bounding_box(ranges)`, `proximity(radius)`, `proximity_hamming(max_dist)` -- implemented for all CoordCube\<N, D, R\> | `geo/src/spatial.rs` |
| DistanceMetrics trait | `hamming_distance`, `euclidean_distance_approx`, `manhattan_distance` -- implemented for all CoordCube\<N, D, R\> | `geo/src/spatial.rs` |
| BoundingBoxIter\<N\> | Iterator over all CoordPath\<N\> in a hyper-rectangle. Mixed-radix enumeration. `count_paths()` for O(1) cardinality | `geo/src/spatial.rs` |
| HammingFilter\<N\> | Iterator adapter over CoordPath\<N\>; yields only paths within a Hamming distance of a center | `geo/src/spatial.rs` |

### tagma-map: hashless string-key map (requires alloc)

| Type | Description | File |
|------|-------------|------|
| CoordKey\<N\> | Fixed N-byte key, type-level length enforcement. Injective to CoordPath | `map/src/coord_gen.rs` |
| DynCoordMap | Dynamic map, ByteWise strategy, all-length strings | `map/src/dyn_coord_map.rs` |
| CoordMap2 | Fixed 2-byte dense map, CoordSpace2 (119 MB), O(1) lookup | `map/src/coord_map2.rs` |
| CoordMapN\<N\> | Fixed N-byte tree map, CoordSpaceN, sparse | `map/src/coord_map_n.rs` |
| CoordMap trait | HashMap-compatible: `insert`, `get`, `remove`, `contains_key` via `&str` | `map/src/coord_map.rs` |
| CoordMapKey\<N\> trait | `_by_coordkey` methods for CoordKey-based access | `map/src/coord_map.rs` |
| CoordCubeMap\<N\> trait | Spatial queries on maps: `proximity` (L∞ radius), `bounding_box_range`. Implemented for CoordMap2, CoordMapN\<N\>, DynCoordMap | `map/src/spatial.rs` |

### tagma-sec: security primitives (requires alloc)

| Type | Description | File |
|------|-------------|------|
| SecStack | Composable security stack over the four modules with switchable `legacy()` and `delos()` implementations | `sec/src/proxy.rs` |
| Authority trait | register, issue (issued epoch and validity window), authorize (Exact/Prefix scope matching with action label), revoke (epoch-scoped) | `sec/src/authority.rs` |
| Integrity trait | seal (record, path, principal, epoch binding), verify, refresh (monotonic re-binding) | `sec/src/integrity.rs` |
| Audit trait | append, verify_chain, prove (inclusion proof), export (evidence bundle) | `sec/src/audit.rs` |
| Channel trait | sign, verify, exchange (receipt binding evidence, remote, epoch), verify_receipt | `sec/src/channel.rs` |

## Quick start

```sh
git clone https://github.com/ssccsorg/syntagma
cd syntagma
./run.sh                # fmt -> clippy -> build -> test -> no_alloc check
```

Or directly:

```sh
cd sw/rust
cargo test --release       # All tests
cargo bench -- spatial     # 18 CoordCube and spatial query benchmarks
cargo bench -- stress      # 500k mixed-operation stress benchmark
cargo bench -- sec         # tagma-sec security layer benchmarks (21)
cargo build --no-default-features  # Verify no_alloc build
```

## Usage

```rust
use tagma_core::{Coord, CoordCube, CoordPath, CoordSet, CoordSpace};

// Compose a coordinate from three axes
let c = Coord::from_axes(5, 10, 15).unwrap();
assert_eq!(c.to_axes(), (5, 10, 15));
assert_eq!(c.to_char(), '걐');  // compositional character display

// Single-character direct-address space (no allocator)
let mut space = CoordSpace::new();
space.place(c, "tagma");
assert_eq!(space.at(&c), Some(&"tagma"));
*space.entry(c).or_insert("default") = "updated";

// Bit-array set
let mut set = CoordSet::new();
set.insert(c);
assert!(set.contains(c));

// CoordCube: interpret 6 characters as 3D space (2 chars per dimension)
let path = CoordPath::<6>::new([
    Coord::new(0).unwrap(), Coord::new(1).unwrap(),  // dim 0
    Coord::new(2).unwrap(), Coord::new(3).unwrap(),  // dim 1
    Coord::new(4).unwrap(), Coord::new(5).unwrap(),  // dim 2
]);
let cube = CoordCube::<6, 3, 2>::from_path(path);
let axis0 = cube.axis(0);  // CoordPath<2> for dimension 0
assert_eq!(axis0.coords()[0].index(), 0);
```

### tagma-geo usage

```rust
use tagma_core::{Coord, CoordCube, CoordPath};
use tagma_geo::{DistanceMetrics, SpatialOps};

// Distance metrics on a 3D cube
let a = CoordCube::<3, 3, 1>::from_path(CoordPath::new([
    Coord::new(1000).unwrap(), Coord::new(2000).unwrap(), Coord::new(3000).unwrap(),
]));
let b = CoordCube::<3, 3, 1>::from_path(CoordPath::new([
    Coord::new(1500).unwrap(), Coord::new(2500).unwrap(), Coord::new(3500).unwrap(),
]));
assert_eq!(a.hamming_distance(&b), 3);
assert!(a.euclidean_distance_approx(&b) > 0.0);
assert_eq!(a.manhattan_distance(&b), 1500);

// Proximity: all paths within L∞ radius 1 of center (9 paths in 2D)
let center = CoordCube::<2, 2, 1>::from_path(CoordPath::new([
    Coord::new(5000).unwrap(), Coord::new(5000).unwrap(),
]));
let nearby: Vec<_> = center.proximity(1).collect();
assert_eq!(nearby.len(), 9);

// Bounding box: enumerate a rectangular region
let ranges = [(0u16, 99u16), (0u16, 99u16)];
let box_paths: Vec<_> = center.bounding_box(&ranges).collect();
assert_eq!(box_paths.len(), 10000);
```

### tagma-map usage

```rust
use tagma_map::{CoordMap, CoordMap2, CoordMapKey, DynCoordMap};
use tagma_map::coord_gen::CoordKey;

// Dynamic map: any non-empty string key
let mut map = DynCoordMap::new();
map.insert("hello", b"world".to_vec());
assert_eq!(map.get("hello"), Some(b"world".to_vec()));

// Fixed 2-byte dense map: 119 MB, O(1), collision-free
let mut map = CoordMap2::new();
map.insert("hi", b"value".to_vec());
assert_eq!(map.get("hi"), Some(b"value".to_vec()));

// Same store, CoordKey-based access
let key = CoordKey::new(*b"hi");
assert_eq!(map.get_by_coordkey(&key), Some(b"value".to_vec()));

// Compile-time key length enforcement
const KEY: CoordKey<2> = CoordKey::from_str_const("hi");

// Contains key, remove, all HashMap-compatible
assert!(map.contains_key("hi"));
map.remove("hi");
```

### CoordCubeMap: spatial queries on maps

```rust
use tagma_map::{CoordMap, CoordMapN, CoordMapKey};
use tagma_map::coord_gen::CoordKey;
use tagma_map::spatial::CoordCubeMap;
use tagma_core::{Coord, CoordCube, CoordPath};
use tagma_geo::SpatialOps;

// Fill a store with 10,000 entries in a 100x100 region
let mut map = CoordMapN::<2>::new();
let fill_center = CoordCube::<2, 2, 1>::from_path(CoordPath::new([
    Coord::new(5000).unwrap(), Coord::new(5000).unwrap(),
]));
for path in fill_center.bounding_box(&[(4950u16, 5050u16), (4950u16, 5050u16)]) {
    map.insert_by_coordkey(&CoordKey::from_coord_path(&path), b"v".to_vec());
}

// Spatial proximity query: find all entries within L∞ radius 1 of center
let center = CoordPath::<2>::new([Coord::new(5000).unwrap(), Coord::new(5000).unwrap()]);
let nearby: Vec<_> = map.proximity::<2, 1>(&center, 1);
// Returns 9 entries (3x3 grid)
assert_eq!(nearby.len(), 9);
```

### tagma-sec usage

```rust
use tagma_core::Coord;
use tagma_sec::proxy::{route_update, SecStack};
use tagma_sec::types::{Path, Scope};

let mut stack = SecStack::delos();
let principal = stack.authority.register(b"coordinator-1");
let scope = Scope::Prefix(vec![Coord::new(1).unwrap(), Coord::new(2).unwrap()]);
let att = stack.authority.issue(principal, scope, 0, 100).unwrap();
let path: Path = vec![
    Coord::new(1).unwrap(),
    Coord::new(2).unwrap(),
    Coord::new(3).unwrap(),
];
let res = route_update(&mut stack, &att, &path, b"route-v1", 50).expect("authorized update");
assert!(stack.channel.verify_receipt(&res.receipt));
```

## Feature matrix

| Feature | `no_alloc` | `alloc` (default) | `mmap` |
|---------|-----------|-------------------|--------|
| Coord | ✅ | ✅ | ✅ |
| CoordPath\<N\> | ✅ | ✅ | ✅ |
| CoordSet | ✅ | ✅ | ✅ |
| CoordCube\<N, D, R\> | ✅ | ✅ | ✅ |
| CoordSpace (inline array) | ✅ | ✅ | ✅ |
| CoordSetN\<N\> (sparse N-d set) | ❌ | ✅ | ❌ |
| CoordSpaceN (heap tree, N>1) | ❌ | ✅ | ❌ |
| CoordSpace2 (dense heap, N=2) | ❌ | ✅ | ❌ |
| CoordSpaceM (mmap dense, N≥3) | ❌ | ❌ | ✅ |
| DynCoordSpace (runtime trie) | ❌ | ✅ | ❌ |
| tagma-geo (spatial ops, metrics) | ❌ | ✅ | ❌ |
| tagma-map (string-key map, HashMap API) | ❌ | ✅ | ❌ |
| tagma-sec (security primitives, route-update workflow) | ❌ | ✅ | ❌ |

## How Tagma works

A Tagma coordinate is computed from three structural axes via the Unicode U+AC00--U+D7AF composition formula defined in ISO/IEC 10646:

$$C(i,m,f) = \text{U+AC00} + 588i + 28m + f, \quad 0 \leq i < 19,\; 0 \leq m < 21,\; 0 \leq f < 28$$

Of 65,536 representable 16-bit states, only 11,172 satisfy this formula. The remaining 54,364 are structurally invalid and hardware-detectable. Each valid value is:

- A 1-D address (Unicode code point) for flat array indexing.
- A 3-D coordinate (initial, medial, final) for structural queries.
- A compositional character for human-readable display.

N-character sequences (CoordPath) extend the address space to $11172^N$ identifiers via direct-index tree traversal. A 6-character identifier covers UUID-scale space; 19 characters match SHA-256's $2^{256}$ identifier space. CoordCube reinterprets these N characters as a D-dimensional grid with R characters per dimension (N = D * R), providing distance metrics and region queries without changing the underlying storage key.

The three-axis composition formula admits unbounded recursive embedding: each axis of a synTagma coordinate can itself be a full CoordPath, enabling physical topology mapping across distributed nodes without modifying the core arithmetic.

## Benchmark: Tagma identity generation (ARMv8.4-A Firestorm)

| Metric | SHA-256 | Tagma (1-syll) | Tagma (6-syll) | Tagma (19-syll) |
|--------|---------|---------------|---------------|----------------|
| Latency | 227 ns/op | 0.38 ns/op | 5.57 ns/op | 54.9 ns/op |
| Speedup | baseline | 597x | 41x | 4.1x |
| Address space | 2^256 | 1.1e4 | 1.9e24 | 2^256 |

CoordSpace2 (N=2 dense heap, 119 MB, `alloc_zeroed`) covers the full 124M 2-character space in a single pre-zeroed allocation -- single load at 0.39 ns, no lazy branching.

## Benchmark: Spatial query vs HashMap (ARMv8.4-A Firestorm)

Same algorithm (iterate + decompose + filter on axis), different memory layout. CoordSpace stores values in contiguous `[Option<V>; 11172]` -- no hash, no collision, no fragmentation. HashMap scatters across buckets.

| Category | Operation | CoordSpace | HashMap | Ratio |
|----------|-----------|-----------|---------|-------|
| Single-op micro | Get single (random coord) | 0.82 ns | 8.50 ns | 10.4x |
| Bulk 11,172 | Insert | 26.4 µs | 385 µs | 14.6x |
| Bulk 11,172 | Get | 6.48 µs | 102 µs | 15.7x |
| Bulk 11,172 | Remove | 15.9 µs | 275 µs | 17.3x |
| Spatial query | Axis filter (medial=10) | 58.0 Melem/s | 24.5 Melem/s | 2.4x |
| Spatial query | CoordSet compound (initial=3 AND medial=5) | 85.0 ns | 11.5 µs | 135x |
| Edge (CS2) | Sparse get 10M | 44.9 ms | 1.05 s | 23.4x |
| Edge (CS2) | Nonexistent prefix (iter scan) | 1.65 ns | 23.05 ms | 14.0Mx |

*Nonexistent prefix (structural vs iter scan): HashMap has no prefix index and must scan all 10M entries to determine that no entry has first coord == 11111. CoordSpace navigates to the branch at that prefix and returns None immediately. The gap (14.0Mx) reflects the difference between structural addressing and content scanning, not between two equivalent hash lookups.*

## Benchmark: CoordCube spatial queries (ARMv8.4-A Firestorm)

CoordCube provides proximity (L∞ Chebyshev radius) and bounding box enumeration by interpreting CoordPath bytes as multi-dimensional coordinates. All values are measured -- previous estimates replaced with real data.

### Overhead vs CoordPath

CoordCube is a zero-cost interpretation layer. Creating a CoordCube from CoordPath costs 0.96 ns once; axis extraction is identical to raw CoordPath access (319 ps vs 319 ps).

### Proximity generation (D=2, R=1, N=2)

| Radius | Paths | Time | Throughput |
|--------|-------|------|------------|
| 0 | 1 | 16.5 ns | 60.6 Melem/s |
| 1 | 9 | 86.6 ns | 103.9 Melem/s |
| 2 | 25 | 126.4 ns | 197.8 Melem/s |
| 3 | 49 | 184.6 ns | 265.5 Melem/s |
| 5 | 121 | 331.4 ns | 365.1 Melem/s |

Baseline (manual loop without CoordCube): 2.51 ns for 9 CoordPath constructions. CoordCube API adds 7.8 ns for iterator infrastructure.

### Bounding box

| Configuration | Paths | Time | Throughput |
|---------------|-------|------|------------|
| N=2, D=2, 100x100 | 10,201 | 14.28 µs | 714 Melem/s |
| N=6, D=6, 3^6 | 729 | 1.70 µs | 427 Melem/s |

### Dimensional scaling (proximity r=2, R=1)

| D | N | Paths | Time | Throughput |
|---|---|-------|------|------------|
| 1 | 1 | 5 | 33.8 ns | 148 Melem/s |
| 2 | 2 | 25 | 126.5 ns | 198 Melem/s |
| 3 | 3 | 125 | 441.9 ns | 283 Melem/s |
| 4 | 4 | 625 | 1.887 µs | 331 Melem/s |

N = D * R is the real driver. Identical throughput at same N (D=2,R=1 vs D=1,R=2 both show ~127 ns).

### Distance metrics (D=3, single pair, runtime-generated coordinates)

| Metric | Latency |
|--------|---------|
| Hamming distance | 1.75 ns |
| Manhattan distance | 2.63 ns |
| Euclidean distance (approx) | 13.5 ns |

Values are from runtime-generated coordinates (PRNG) to prevent compile-time constant folding. The 3.2 ps shown in earlier runs was an artifact of pre-computation.

## Benchmark: CoordCube + CoordMap proximity (ARMv8.4-A Firestorm)

End-to-end spatial queries combining CoordCube path generation with map store lookup:

| Scenario | Store type | Query | Latency | Found |
|----------|-----------|-------|---------|-------|
| Sequential (9 manual lookups) | CoordMapN\<2\> | Tree+Path | 158 ns | 9 |
| CoordCube proximity r=1 | CoordMapN\<2\> | Tree+Cube | 285 ns | 9 |
| CoordCube proximity r=2 | CoordMapN\<2\> | Tree+Cube | 626 ns | 25 |
| CoordCube proximity r=5 | CoordMapN\<2\> | Tree+Cube | 2.55 µs | 121 |
| CoordCube proximity r=1 | CoordMap2 (dense) | Dense+Cube | 282 ns | 9 |
| CoordCube proximity r=1 | DynCoordMap | Cube | 161 ns | 9 |
| CoordCube proximity r=2 | DynCoordMap | Cube | 290 ns | 25 |
| CoordCube proximity r=1 | CoordMapN\<2\> sparse | Cube | 48.5 ns | 9 |
| CoordCube proximity r=1 | CoordMapN\<2\> empty | Cube | 15.7 ns | 0 |
| Sequential (9 lookups) | DynCoordMap | Baseline | 259 ns | 9 |

Breakdown of the 127 ns overhead (Tree+Path 158 ns -> Tree+Cube 285 ns):
- Vec allocation: 37 ns
- Vec::push x9: 74 ns
- Path generation: 16 ns

Key insight: On tree stores, the extra overhead is dominated by Vec allocation and push, not coordinate arithmetic. Dense vs tree backend makes almost no difference (282 ns vs 285 ns) because Vec push dominates. On sparse stores, CoordCube is up to 3.3x faster than sequential lookups because it avoids tree lookups for nonexistent paths. On DynCoordMap, proximity is 1.6x faster than sequential (161 vs 259 ns) due to higher per-lookup cost.

## Benchmark: Compound axis query via CoordSet (N=1 bit array, pre-computed)

Pre-computed per-axis bit sets (19 initial + 21 medial, each 1.4 KB) answer compound axis queries with a single bitwise AND.

| Implementation | Time | Throughput | vs HashMap |
|---------------|------|------------|------------|
| CoordSet bitwise AND | 85.7 ns | 327 Melem/s | **144x** |
| HashMap iterate+filter | 12.3 µs | 2.28 Melem/s | baseline |

## Benchmark: CoordSetN set operations (N=2, sparse tree)

Set operations on 500-element sets with 250-element overlap. CoordSetN\<2\> uses a heap-allocated bit tree (per-node bit array), not a single flat bit array -- unlike CoordSet (N=1, 1.4 KB inline).

| Operation | CoordSetN\<2\> | HashSet |
|-----------|---------------|---------|
| Union 500 + 500 | 4.57 ms | 29.9 µs |
| Intersection 500 + 500 | 1.91 ms | -- |
| Difference 500 + 500 | 1.91 ms | -- |
| Is subset 500 + 500 | 49.0 ns | -- |
| Is disjoint 500 + 500 | 910 µs | 11.8 ns |

CoordSetN union/intersection/difference are tree traversal operations (not bitwise AND), which makes them slower than HashSet at N=2. The advantage of CoordSetN appears at higher dimensions where HashSet keys grow linearly with N while the tree structure stays compact.

## Benchmark: tagma-map vs HashMap (ARMv8.4-A Firestorm)

tagma-map is a hashless map store: it converts `&str` to Coord sequences instead of hashing them. The critical question is whether this conversion is faster than SipHash-2-4.

### Single operation

| Variant | Insert | Get | Contains |
|---------|--------|-----|----------|
| **CoordMap2** (fixed 2B) | **18.7 ns** | **22.1 ns** | **21.7 ns** |
| CoordMapN\<2\> (fixed 2B) | 18.9 ns | 21.7 ns | 21.7 ns |
| DynCoordMap (variable) | 49.4 ns | 42.4 ns | 42.4 ns |
| **HashMap\<String\>** | 44.9 ns | 23.8 ns | 13.0 ns |

CoordMap2 get is 1.08x faster than HashMap. The difference (21.67 ns) is the str-to-CoordKey conversion cost plus Vec clone; the slot load itself is 0.39 ns.

### Three-scale workload (get, per-op ns)

| Variant | 10k ops | 1M ops | 10M ops | Trend |
|---------|---------|--------|---------|-------|
| **CoordMap2** | **22.0 ns** | **21.4 ns** | **21.5 ns** | **flat** |
| CoordMapN\<2\> | 22.7 ns | 21.9 ns | 22.1 ns | flat |
| DynCoordMap | 55.8 ns | 57.4 ns | 60.6 ns | +7% |
| **HashMap\<String\>** | 21.9 ns | 24.2 ns | 23.8 ns | **+19%** |

CoordMap2 latency is scale-invariant: 22.0 ns at 10k, 21.5 ns at 10M. HashMap per-op cost rises 19% from 10k to 10M as the working set exceeds cache capacity.

### Contains key (per-op ns)

| Variant | 10k ops | 1M ops | 10M ops |
|---------|---------|--------|---------|
| CoordMap2 | 22.3 ns | 21.6 ns | 21.6 ns |
| CoordMapN\<2\> | 23.2 ns | -- | -- |
| DynCoordMap | 54.7 ns | -- | -- |
| HashMap | 13.2 ns | 19.9 ns | 19.9 ns |

HashMap's bool-return advantage is erased by cache pressure at scale.

Once data is in Tagma coordinate space, spatial capabilities (prefix scan, axis filter, proximity, bounding box, distance metrics) are available at zero additional conversion cost.

## Benchmark: tagma-sec security layer (ARMv8.4-A Firestorm)

tagma-sec provides authority, integrity, audit, and channel primitives for coordination traffic. Security rests on keyed primitives (blake3) over public coordinate arithmetic: coordinate arithmetic is negligible, keyed hashing dominates. Measured values come from the `sec` criterion group in `sw/rust/benches/bench.rs`.

| Scenario | Benchmark | Measured | Insight |
|----------|-----------|----------|---------|
| Path authorization | `authorize/allow` | 36.8 ns | O(scope depth), independent of store size |
| Fail-closed rejection | `authorize/deny` | 3.1 ns | Scope miss short-circuits at the first coord, ~12x cheaper than Allow |
| Revoked scope | `authorize/revoked` | 40.5 ns | One map lookup on top of the scope match |
| Epoch replay detection | `seal/pattern` | 171.4 ns | Epoch and principal binding costs ~33 ns over the 2-way seal (138.8 ns) |
| Seal verification | `verify/pattern` | 129.2 ns | Recompute and compare |
| Audit chaining | `audit/verify_chain` (10k) | 9.9 µs | ~1 ns per entry prev-link walk, no hashing |
| Offline investigation | `audit/prove`, `audit/export` (10k) | ~200 µs | Memory-bound clones, ~20 ns per entry |
| Receipt verification | `channel/verify_receipt` | 86.8 ns | Parsing plus recompute, same order as verify |
| Route update workflow | `route_update/pattern` | 696.1 ns | Exact sum of the module costs (691 ns), cost-transparent composition |
| Baseline contrast | `route_update/legacy` | 609.5 ns | The tagma-sec pattern adds ~87 ns per update, the price of replay detection |

The route-update workflow is a cost-transparent composition: 696.1 ns equals the sum of its module calls (authorize 36.8, seal 171.4, verify 129.2, append 110.3, append 110.3, exchange 132.7), so the proxy and trait-object dispatch add no measurable overhead. Deny is ~12x cheaper than allow, so fail-closed rejection is cheap. Chain verification is ~1 ns per entry while prove and export are ~20 ns per entry and memory-bound, so offline investigation scales with evidence size rather than log size.

## Documentation

- [synTagma project page](https://docs.ssccs.org/projects/syntagma/) -- Project overview, paradigm shift, papers
- [White Paper](https://docs.ssccs.org/projects/syntagma/tagma) -- Tagma coordinate space, decoder, hardware implementation, benchmarks
- [synTagma coordination layer](https://docs.ssccs.org/projects/syntagma/) -- Recursive topology mapping, transport, distributed resolver, consistency
- [Tagma-ID](https://docs.ssccs.org/projects/syntagma/tagma/id) -- Content-addressable identity without hash functions
- [Specification](docs/spec/coord-space.md) -- Language-independent Tagma coordinate space definition
- [Specification (tagma-sec)](docs/spec/tagma-sec.md) -- Security layer: authority, integrity, audit, channel, hybrid confidentiality
- [Hardware verification](hw/README.md) -- RTL decoder, exhaustive verification (11,172 vectors, formal equivalence), FPGA PnR, Sky130 standard cell report
- [Rustdoc (tagma-core)](https://docs.ssccs.org/projects/syntagma/tagma/core/) -- Coord, CoordPath, CoordSpace, CoordSpaceN, CoordCube, DynCoordSpace
- [Rustdoc (tagma-geo)](https://docs.ssccs.org/projects/syntagma/tagma/geo/) -- SpatialOps, DistanceMetrics, BoundingBoxIter, HammingFilter
- [Rustdoc (tagma-map)](https://docs.ssccs.org/projects/syntagma/tagma/map/) -- CoordMap, CoordMap2, CoordMapN, DynCoordMap, CoordCubeMap, CoordKey
- [Rustdoc (tagma-sec)](https://docs.ssccs.org/projects/syntagma/tagma/sec/) -- SecStack, Authority, Integrity, Audit, Channel

## License

Apache 2.0 -- see [LICENSE](LICENSE).
