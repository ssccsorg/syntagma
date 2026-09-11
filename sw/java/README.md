# tagma-java

Java 21 port of the Tagma coordinate engine under `sw/java`. The C++ port in
`sw/cpp` is the primary reference specification; the Rust sources in
`sw/rust` remain the underlying specification where the C++ port defers or
leaves a semantic gap. Java mirrors the module layout: the same types, the
same semantics, verified by JUnit 5 tests translated from the reference test
suites.

## Scope

Every module of the reference implementations is covered.

| Module | Types |
|--------|-------|
| `tagma-core` | `Coord`, `CoordPath`, `CoordSet`, `CoordSpace<V>`, `CoordCube`, `CoordSpaceN`, `CoordSetN`, `DynCoordSpace`, `CoordSpaceM` (anonymous off-heap) |
| `base11172` | `Base11172` serialization |
| `tagma-geo` | `BoundingBoxIter`, `HammingFilter`, `SpatialOps` (proximity, bounding box, Hamming filtering, distance metrics, dimension values) |
| `tagma-map` | `CoordKey`, the coordinate generation strategies, `CoordMap`, `CoordMapKey`, `CoordPathLookup`, `CoordMapN`, `CoordMap2`, `DynCoordMap`, `CoordCubeMap` |
| `tagma-sec` | the hash layer, `Scope`, `Attestation`, `Seal`, `Event`, `SignedEvidence`, `Receipt`, `Authority`, `Integrity`, `Channel`, `LegacyAuthority`, `DelosAuthority`, `SecStack`, `Audit` |
| `bench` | `tagma-bench`, the benchmark suite mirroring `sw/cpp/bench/bench.cpp` |

## Layout

```
sw/java
├── pom.xml                  aggregator (Java 21, JUnit 5, -Xlint:all)
├── run.sh                   module entry point: mvn verify, --bench
├── tagma-core/              artifact org.ssccs.syntagma:tagma-core
├── base11172/               artifact org.ssccs.syntagma:base11172
├── tagma-geo/               artifact org.ssccs.syntagma:tagma-geo
├── tagma-sec/               artifact org.ssccs.syntagma:tagma-sec
├── tagma-map/               artifact org.ssccs.syntagma:tagma-map
└── bench/                   artifact org.ssccs.syntagma:tagma-bench
```

The root `run.sh` runs the Java build through `check_java` whenever `mvn` and
`java` are available, and `run.sh --bench` additionally runs the Java
benchmark suite through `check_java_bench`. The CI workflows `java` (build and
test) and `java-bench` (benchmark suite) exercise `sw/java` with JDK 21
(temurin).

## Byte-space map domain

The store key space is one byte per character, so map queries operate in the
per-character domain `[0, 256)`.

- `CoordKey.BYTE_DOMAIN` is 256, and `CoordKey.fromCoordPath` rejects a
  character index that reaches or exceeds it.
- `CoordCubeMap.proximity` and `CoordCubeMap.boundingBoxRange` reject an
  out-of-domain center and out-of-domain range bounds before generation runs.
- Proximity generation goes through
  `SpatialOps.proximityBounded(cube, radius, CoordKey.BYTE_DOMAIN)`, so a
  radius that crosses the domain edge clamps at the edge, and the result
  capacity saturates on overflow.

A byte key cannot carry a `Coord` index above 255, and folding such an index
onto its low byte collides distinct entries (0 and 256). That defect is
recorded as issue #59. `ProximityBoundedTest`, `CoordKeyTest`,
`CoordCubeMapTest` and `DensityWindowTest` pin the boundary behavior,
including the domain-edge clamp at byte 254 with radius 5 over domain 256
(49 paths), the domain floor clamp at byte 5 (121 paths), the full-domain
clamp at index 11171 with radius 3 (16 paths), the nine-entry sparse fill
returning one hit, and the rejection of a path index of 256.

## Parity and porting differences

| C++ reference | Java port | Notes |
|---------------|-----------|-------|
| `tagma::Coord` | `record Coord` | `Coord` wraps the raw index with a validating canonical constructor; invalid values are reachable only through the `Optional`-returning factories |
| `Coord::axes()` -> tuple | `Coord.axes()` -> `Coord.Axes` record | The third component is `final_` because `final` is a reserved word |
| `CoordPath<N>` | `CoordPath` | Java has no const generics or template parameters. The compile-time length tag becomes immutable instance state; `CoordPath` copies its array for value semantics |
| `CoordCube<N, D, R>` | `CoordCube` | `N`, `D`, `R` are instance state; `CoordCube.fromPath(d, r, path)` throws `IllegalArgumentException` when `N != D * R`, mirroring the Rust runtime assertion (the C++ `static_assert` is a compile-time equivalent) |
| `CoordSet` (`std::bitset`) | `CoordSet` (`java.util.BitSet`) | Same observable operations; `size()`, `copy()`, `capacity()` |
| `CoordSpace<V>` | `CoordSpace<V>` | Java stores the slot array on the heap; null values are rejected, mirroring `Option<V>` / `std::optional<V>` |
| `CoordSpace::Entry::or_insert` -> `V&` | `Entry.orInsert` -> `ValueRef<V>` | `ValueRef` is the Java counterpart of `V&` / `&mut V`: `get`, `set`, `update` write through to the slot |
| `CoordSpaceN<N, V>` | `CoordSpaceN` | Depth and value width are instance state; a wrong path length, an out-of-range depth and set operations across depths throw `IllegalArgumentException` |
| `CoordSetN<N>` | `CoordSetN` | Same instance-state mapping; content equality mirrors the C++ `operator==` |
| `DynCoordSpace<V>` | `DynCoordSpace` | Depth-flexible space with the same placement and iteration surface |
| `CoordSpaceM<N, V>` (anonymous `mmap`) | `CoordSpaceM` (off-heap direct buffers) | The references reserve the whole slot region with one anonymous mapping and let the kernel commit pages on write. Java has no portable anonymous mapping, so the port materializes the region window by window with `ByteBuffer.allocateDirect` on first touch, each window sized to the largest multiple of the slot stride that fits a direct buffer. The space is anonymous and in-memory only: it owns no file, no header and no on-disk format, because materialization, layouts and formats belong to chton. Values are limited to the fixed-width primitives selected by a `Class<V>` token, because a direct buffer cannot hold arbitrary objects. The buffers are accounted against `-XX:MaxDirectMemorySize`, and a placement materializes its whole window where the reference commits only the touched pages |
| `tagma_geo` free functions | `SpatialOps` static facade | Java cannot add methods to `CoordCube`, so the Rust `SpatialOps` trait and the C++ free functions become static methods |
| `BoundingBoxIter<N>::count_paths` | `BoundingBoxIter.countPaths()` -> `long` | Saturates at `Long.MAX_VALUE` where the references saturate at `SIZE_MAX` / `usize::MAX`; the `N == 0` case returns 0 |
| `HammingFilter<N>` | `HammingFilter` | The constructor skip becomes a `hasNext()` look-ahead with single-element buffering |
| `CoordKey<N>` | `CoordKey` | The key length becomes instance state; `BYTE_DOMAIN` carries the domain contract, and `bytes()` returns a defensive copy where the C++ returns a const reference |
| `type DefaultDynamic = ByteWise` | `CoordGen.DEFAULT_DYNAMIC` | Java has no type aliases, so the alias is the interface constant holding `ByteWise.INSTANCE` |
| `CoordMap2` | `CoordMap2` | `CoordMapN` fixed at depth 2 over the lazy `CoordSpaceN<2>` tree, following the C++ reference; the Java core has no dense depth-2 space |
| blake3 keyed hashing (Rust) | SHA-256 and RFC 2104 HMAC-SHA-256 | The C++ port already made this substitution and documented it; Java uses `MessageDigest` and `Mac`, the produced bytes are identical to C++, and both ports pin module-level seal, receipt, channel and audit vectors that were computed with openssl |
| `tagma_sec` types | records and final classes | `std::optional<T>` maps to `Optional<T>` / `OptionalLong`; 32-byte tags and payloads are copied in and cloned out |
| `tagma_bench` (`--json`, `--commit`, `--timestamp`) | `tagma-bench` (same flags) | The harness warms up to a steady state, because the JIT only optimizes code that has already run (a one-second floor with five-percent agreement and two settling rounds), and it pins `-XX:MaxDirectMemorySize=3g` because the mapped space materializes a window on first touch; `--quick`, `--iterations` and `--rounds` remain for smoke runs |
| `DynCoordMap` iterator keys (`std::string` of the raw bytes) | `Map.Entry<CoordKey, byte[]>` | A Java `String` re-encodes as UTF-8 on the way back into the store and would not round-trip a non-ASCII key, so the iterator yields the validated byte key instead |
| `CoordPathLookup` (the depth lives in the type) | a length mismatch is an absent path | The references cannot express the mismatch; the Java lookup returns `Optional.empty()`, while the coordinate-key surface keeps the runtime rejection of the underlying space |

Method names follow Java conventions (`size`, `isEmpty`, `copy`, `union`)
and map one-to-one onto the C++/Rust names documented in each class.

### Reference choice

Rust ownership, borrowing, and const generics have no Java counterpart and
require a redesign pass. The C++ port already expresses the same semantics in
an imperative, object-oriented language, so it maps to Java with less
distortion. Each class javadoc names the mirrored C++ header and the Rust
source file.

### Java runtime design notes

`Coord` is a record over a single `int` index so the type stays a plain
value with no identity semantics and no hidden state; if Project Valhalla
value classes arrive, this record converts without structural change. Axis
math, code point conversion, and Hamming distance are plain integer
arithmetic, which the JIT can inline and scalarize. Iteration avoids
per-element temporary objects where the reference semantics allow it: the
`CoordSet` iterator scans the live bit set instead of materializing a
snapshot, and `CoordSpace` exposes a snapshot `entries()` list only where
the C++/Rust tests require an iteration surface.

## Verification

```sh
# Full build + test of the reactor (root run.sh entry):
./run.sh --check        # requires JDK 21 + Maven on PATH

# Direct:
cd sw/java && ./run.sh  # or: mvn -B verify (from sw/java)

# Benchmark suite (writes bench/result/bench-<timestamp>-<commit>.json):
cd sw/java && ./run.sh --bench
```

The reactor holds 342 tests, all green.

| Module | Tests | Translated from |
|--------|-------|-----------------|
| `tagma-core` | 115 | `test_coord.cpp`, `test_core_types.cpp`, `test_coord_cube.cpp`, `test_tree_types.cpp`, `test_coord_space_m.cpp`, plus the Rust-only cases for `DynCoordSpace` |
| `base11172` | 5 | `test_base11172.cpp` |
| `tagma-geo` | 46 | `test_spatial.cpp` and the integration suite `sw/rust/geo/tests/spatial_window.rs` |
| `tagma-sec` | 54 | `test_workflow.cpp`, `test_delos.cpp`, `test_scenarios.cpp`, plus the openssl-pinned hash and module-level tag vectors |
| `tagma-map` | 72 | `test_map.cpp`, `test_cube_map.cpp`, `test_dyn_map.cpp` and the integration suite `sw/rust/map/tests/density_window.rs` |
| `bench` | 50 | harness coverage: CLI parsing, statistics, JSON shape, warmup policy and profile handling |

Rust-only behaviors that the C++ port does not expose are documented as
follow-ups rather than invented API surface. The port provides `copy`,
structural `equals`/`hashCode`, `entriesPrefix` and `Entry.andModify` where
the C++ omissions would have blocked a translated case, and leaves the
`CoordSpaceN<1>` member set, `FromIterator` construction and `Drain` out of
the surface. The distance-metric functions mirror the reference code, which
wraps the dimension maximum modulo 2^64 for large resolutions while the
accompanying comments claim zero; the javadoc states the modulo wrap and no
test locks the wrapped value.

## Reference

- C++ port: `sw/cpp` (`tagma_core`, `base11172`, `tagma_geo`, `tagma_map`, `tagma_sec`, `bench`)
- Rust reference: `sw/rust` (`core`, `base11172`, `geo`, `map`, `sec`, `benches`)
