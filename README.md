# synTagma

synTagma is a spatial coordinate space computing system built on Tagma, a 16-bit coordinate primitive embedded in the Unicode Hangul syllable block (U+AC00--U+D7AF). Every valid 16-bit value decomposes into three independent axes (initial, medial, final), serving simultaneously as a 1-D address and a 3-D coordinate. The reference implementation is a `#![no_std]` Rust library.

Tagma is a primitive where the address is the coordinate — not a flat pointer, but a point in an N-dimensional geometric space. This is made possible by a 16-bit Unicode block allocated to a 3-axis writing system, which provides a collision-free, hash-less, structurally addressable coordinate space.

## Layers

```
synTagma (system)
  └─ Coordination layer (protocol, topology, distributed resolver)
  └─ Tagma core primitive (Coord, CoordPath, CoordSet, CoordSpace)
```

- Tagma — the core primitive: a 16-bit structural coordinate with closed-form composition, zero collisions, and single-cycle combinational decoding. The atomic identity primitive.
- synTagma coordination layer — recursive coordinate space expansion, physical topology mapping, distributed routing, and consistency protocol. Defined in the [synTagma](https://docs.ssccs.org/projects/syntagma/tagma/syn.html).

## Tagma primitive: Feature levels

Tagma provides a single feature gate: `alloc` (default: on). Without it (`--no-default-features`), all Tagma types are `no_std` + `no_alloc`.

| Level | Feature flags | Heap | Available types |
|-------|---------------|------|-----------------|
| no_alloc | (none) | Never | Coord, CoordPath, CoordSet, CoordSpace |
| alloc | `alloc` (default) | Optional | + CoordSpaceN\<N\>, DynCoordSpace |

## Tagma type reference

### Always available (no_std, no allocator)

| Type | Description | File |
|------|-------------|------|
| Coord | 16-bit atomic coordinate (0..11172), 3-axis composition/decomposition, Hamming distance, Hangul display | `core/src/coord.rs` |
| CoordPath\<N\> | Index path (not a hash key), compile-time N-element Coord array | `core/src/coord_path.rs` |
| CoordSet | Bit array over 11,172 slots (1.4 KB). Union, intersection, difference, subset tests, `Copy` | `core/src/coord_set.rs` |
| CoordSpace\<V\> | Single-syllable direct-address table. Inline `[Option<V>; 11172]` — zero heap. O(1), no hashing, no collisions | `core/src/coord_space.rs` |
| base11172 | Self-validating serialization: arbitrary bytes to Hangul syllable strings | `base11172/src/lib.rs` |

Test coverage: 170 unit/integration tests + 15 doc-tests, all passing. Zero clippy warnings. CI runs `cargo fmt --check`, `cargo clippy`, `cargo build --release`, `cargo test --release`, `cargo build --no-default-features` (no_alloc verification).

### Requires alloc (default feature)

| Type | Description | File |
|------|-------------|------|
| CoordSpaceN\<N, V\> | N-level direct-address tree. Lazy heap allocation per node. `N` dereferences per lookup | `core/src/coord_space_n.rs` |
| CoordSpace2\<V\> | 2-syllable ($1.25 \times 10^8$ space). Type alias for `CoordSpaceN<2, V>` | `core/src/coord_space_n.rs` |
| CoordSpace6\<V\> | 6-syllable UUID-scale ($1.94 \times 10^{24}$). Type alias for `CoordSpaceN<6, V>` | `core/src/coord_space_n.rs` |
| CoordSpace12\<V\> | 12-syllable ($2.41 \times 10^{67}$). Type alias for `CoordSpaceN<12, V>` | `core/src/coord_space_n.rs` |
| CoordSpace19\<V\> | 19-syllable ($\approx 2^{256}$, SHA-256-scale). Type alias for `CoordSpaceN<19, V>` | `core/src/coord_space_n.rs` |
| DynCoordSpace\<V\> | Variable-depth trie, `&[Coord]` runtime path. Mixed-depth slot (Both) preserves shallow values | `core/src/dyn_coord_space.rs` |

## Quick start

```sh
git clone https://github.com/ssccsorg/syntagma
cd syntagma
./run.sh                # fmt → clippy → build → test → no_alloc check
```

Or directly:

```sh
cd sw/rust
cargo test --release       # All tests
cargo bench -- stress      # 500k mixed-operation stress benchmark
cargo build --no-default-features  # Verify no_alloc build
```

## Usage

```rust
use tagma_core::{Coord, CoordSpace, CoordSet};

// Compose a coordinate from three axes
let c = Coord::from_axes(5, 10, 15).unwrap();
assert_eq!(c.to_axes(), (5, 10, 15));
assert_eq!(c.to_char(), '걐');  // Hangul syllable display

// Single-syllable direct-address space (no allocator)
let mut space = CoordSpace::new();
space.place(c, "tagma");
assert_eq!(space.at(&c), Some(&"tagma"));
*space.entry(c).or_insert("default") = "updated";

// Bit-array set
let mut set = CoordSet::new();
set.insert(c);
assert!(set.contains(c));
```

## Feature matrix

| Feature | `no_alloc` | `alloc` (default) |
|---------|-----------|-------------------|
| Coord | ✅ | ✅ |
| CoordPath\<N\> | ✅ | ✅ |
| CoordSet | ✅ | ✅ |
| CoordSpace (inline array) | ✅ | ✅ |
| CoordSpaceN (heap tree, N>1) | ❌ | ✅ |
| DynCoordSpace (runtime trie) | ❌ | ✅ |

## How Tagma works

A Tagma coordinate is computed from three structural axes via the Hangul composition formula defined in ISO/IEC 10646:

$$C(i,m,f) = \text{U+AC00} + 588i + 28m + f, \quad 0 \leq i < 19,\; 0 \leq m < 21,\; 0 \leq f < 28$$

Of 65,536 representable 16-bit states, only 11,172 satisfy this formula. The remaining 54,364 are structurally invalid and hardware-detectable. Each valid value is:

- A 1-D address (Unicode code point) for flat array indexing.
- A 3-D coordinate (initial, medial, final) for structural queries.
- A Hangul syllable for human-readable display.

N-syllable sequences (CoordPath) extend the address space to $11172^N$ identifiers via direct-index tree traversal. A 6-syllable identifier covers UUID-scale space; 19 syllables match SHA-256's $2^{256}$ identifier space.

The three-axis composition formula admits unbounded recursive embedding: each axis of a synTagma coordinate can itself be a full CoordPath, enabling physical topology mapping across distributed nodes without modifying the core arithmetic.

## Benchmark: Tagma identity generation (Apple M1)

| Metric | SHA-256 | Tagma (1-syll) | Tagma (6-syll) | Tagma (19-syll) |
|--------|---------|---------------|---------------|----------------|
| Latency | 227 ns/op | 0.38 ns/op | 5.57 ns/op | 54.9 ns/op |
| Speedup | baseline | 597x | 41x | 4.1x |
| Address space | 2^256 | 1.1e4 | 1.9e24 | 2^256 |

## Benchmark: Spatial query vs HashMap (Apple M1)

Same algorithm (iterate + decompose + filter on axis), different memory layout. CoordSpace stores values in contiguous `[Option<V>; 11172]` — no hash, no collision, no fragmentation. HashMap scatters across buckets.

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

## Documentation

- [synTagma project page](https://docs.ssccs.org/projects/syntagma/) — Project overview, paradigm shift, papers
- [White Paper](https://docs.ssccs.org/projects/syntagma/tagma/wp.html) — Tagma coordinate space, decoder, hardware implementation, benchmarks
- [synTagma coordination layer](https://docs.ssccs.org/projects/syntagma/tagma/syn.html) — Recursive topology mapping, transport, distributed resolver, consistency
- [Tagma-ID](https://docs.ssccs.org/projects/syntagma/tagma/id.html) — Content-addressable identity without hash functions
- [Specification](spec/coord-space.md) — Language-independent Tagma coordinate space definition
- Rustdoc — `cargo doc --no-deps -p tagma-core` for API reference

## License

Apache 2.0 — see [LICENSE](LICENSE).
