# Tagma

**Content-addressable structural primitive defined by the Unicode Hangul syllable block.**

Tagma replaces hash-based identity generation with direct structural addressing over a fixed 16-bit coordinate space. Every valid 16-bit value in the Hangul syllable block (U+AC00--U+D7AF) decomposes into three independent axes (initial, medial, final), serving simultaneously as a 1-D address and a 3-D coordinate. The reference implementation is a `#![no_std]` Rust library.

## Feature levels

Tagma provides a single feature gate: `alloc` (default: on). Without it (`--no-default-features`), all types are `no_std` + `no_alloc` — no heap allocator required, compatible with bare-metal MCUs and embedded targets.

| Level | Feature flags | Heap | Available types |
|-------|---------------|------|-----------------|
| **no_alloc** | (none) | Never | Coord, CoordPath, CoordSet, CoordFlatMap (CoordMap), CoordKey trait |
| **alloc** | `alloc` (default) | Optional | + CoordTreeMap\<N\>, DynCoordMap, CoordKeyMap, CoordKey impls for String |

## Type reference

### Always available (no_std, no allocator)

| Type | Description | File |
|------|-------------|------|
| **Coord** | 16-bit atomic coordinate (0..11172), 3-axis composition/decomposition, Hamming distance, Hangul display | `core/src/coord.rs` |
| **CoordPath\<N\>** | Index path (not a hash key), compile-time N-element Coord array. `From<Coord>`, `From<[Coord; N]>` | `core/src/path.rs` |
| **CoordSet** | Bit array over 11,172 slots (1.4 KB). Union, intersection, difference, subset tests, `Copy` | `core/src/set.rs` |
| **CoordMap\<V\>** ($\equiv$ CoordFlatMap) | Single-syllable direct-address table. Inline `[Option<V>; 11172]` — zero heap. O(1), no hashing, no collisions | `core/src/flat.rs` |
| **CoordKey\<N\>** (trait) | Converts application key types to CoordPath for direct addressing. Implemented for `Coord`, `&str`, `&[u8]`, `u128`, `[u8; 16]`, `[u8; 32]` | `core/src/key.rs` |

### Requires alloc (default feature)

| Type | Description | File |
|------|-------------|------|
| **CoordTreeMap\<N, V\>** | N-level direct-address tree. Lazy heap allocation per node. `N` dereferences per lookup | `core/src/map.rs` |
| **CoordMap2\<V\>** | 2-syllable ($1.25 \times 10^8$ space). Type alias for `CoordTreeMap<2, V>` | `core/src/map.rs` |
| **CoordMap6\<V\>** | 6-syllable UUID-scale ($1.94 \times 10^{24}$). Type alias for `CoordTreeMap<6, V>` | `core/src/map.rs` |
| **CoordMap12\<V\>** | 12-syllable ($2.41 \times 10^{67}$). Type alias for `CoordTreeMap<12, V>` | `core/src/map.rs` |
| **CoordMap19\<V\>** | 19-syllable ($\approx 2^{256}$, SHA-256-scale). Type alias for `CoordTreeMap<19, V>` | `core/src/map.rs` |
| **DynCoordMap\<V\>** | Variable-depth trie, `&[Coord]` runtime path. Mixed-depth slot (Both) preserves shallow values | `core/src/dyn_coord.rs` |
| **CoordKeyMap\<N, K, V\>** | HashMap-compatible wrapper over CoordTreeMap. `insert(key, value)`, `get(&key)` — identical to std HashMap | `core/src/keymap.rs` |

## Quick start

```sh
git clone https://github.com/ssccsorg/tagma
cd tagma
./run.sh                # fmt → clippy → build → test → no_alloc check
```

Or directly:

```sh
cd sw/rust
cargo test --release       # All 187 tests
cargo bench -- stress      # 500k mixed-operation stress benchmark
cargo build --no-default-features  # Verify no_alloc build
```

## Usage

```rust
use tagma_core::{Coord, CoordKeyMap, CoordMap, CoordSet};

// Compose a coordinate from three axes
let c = Coord::from_axes(5, 10, 15).unwrap();
assert_eq!(c.to_axes(), (5, 10, 15));
assert_eq!(c.to_char(), '걐');  // Hangul syllable display

// Hash-free map — identical API to std HashMap
let mut map: CoordKeyMap<1, &str, &str> = CoordKeyMap::new();
map.insert("user:42", "tagma");
assert_eq!(map.get(&"user:42"), Some(&"tagma"));
*map.entry("counter").or_insert("0") = "1";

// Single-syllable direct-address (no allocator)
let mut flat = CoordMap::new();
flat.insert(c, "zero heap");
assert_eq!(flat.get(&c), Some(&"zero heap"));

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
| CoordMap (inline array) | ✅ | ✅ |
| CoordKey trait | ✅ | ✅ |
| CoordTreeMap\<N\> (heap tree) | ❌ | ✅ |
| DynCoordMap (runtime trie) | ❌ | ✅ |
| CoordKeyMap (HashMap API) | ❌ | ✅ |
| CoordKey for String | ❌ | ✅ |
| base11172 serialization | ❌ | ✅ |

## How it works

A Tagma coordinate is computed from three structural axes via the Hangul composition
formula defined in ISO/IEC 10646:

$$C(i,m,f) = \text{U+AC00} + 588i + 28m + f, \quad 0 \leq i < 19,\; 0 \leq m < 21,\; 0 \leq f < 28$$

Of 65,536 representable 16-bit states, only 11,172 satisfy this formula. The remaining
54,364 are structurally invalid and hardware-detectable. Each valid value is:

- A 1-D address (Unicode code point) for flat array indexing.
- A 3-D coordinate (initial, medial, final) for structural queries.
- A Hangul syllable for human-readable display.

N-syllable sequences (CoordPath) extend the address space to $11172^N$ identifiers
via direct-index tree traversal. A 6-syllable identifier covers UUID-scale space;
19 syllables match SHA-256's $2^{256}$ identifier space.

The three-axis composition formula admits unbounded recursive embedding: each axis
of a SynTagma can itself be a full CoordPath. The SynTagma specification defines
how this recursive structure is mapped onto physical topologies.

## Benchmark

On Apple M1 (development platform):

```
Tagma (1-syll):         2 ns/op    (space: 1.1e4)
Tagma (6-syll):        11 ns/op    (space: 1.9e24, UUID-scale)
Tagma (19-syll):       35 ns/op    (space: 2^256)
SHA-256:              227 ns/op

Speedup vs SHA-256:
  1-syll:   115x
  6-syll:    21x
  19-syll:    6.5x
```

On GitHub CI (x86_64, ubuntu-latest) for reproducibility:

```
Tagma (1-syll):        20 ns/op
Tagma (6-syll):       153 ns/op
Tagma (19-syll):      456 ns/op
SHA-256:             4437 ns/op
```

## Documentation

- **[White Paper](https://docs.ssccs.org/projects/tagma/paper/wp.html)** — Full technical analysis: coordinate space, decoder, hardware implementation, benchmarks
- **[SynTagma](https://docs.ssccs.org/projects/tagma/paper/syn.html)** — External coordination layer: physical topology mapping, transport, distributed resolver, consistency model
- **[Master Document](docs/index.qmd)** — Project overview, paradigm shift, core data structures
- **[Specification](spec/coord-space.md)** — Language-independent coordinate space definition
- **Rustdoc** — `cargo doc --no-deps -p tagma-core` for API reference

## License

Apache 2.0 — see [LICENSE](LICENSE).
