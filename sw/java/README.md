# tagma-java

Java 21 port of the Tagma coordinate engine under `sw/java`. The C++ port in
`sw/cpp` is the primary reference specification; the Rust sources in
`sw/rust` remain the underlying specification where the C++ port defers or
leaves a semantic gap. Java mirrors the module layout: the same types, the
same semantics, verified by JUnit 5 tests translated from the C++ test
suites.

## Scope

Current scope covers milestones M1 to M3 of issue #60: the no-alloc
coordinate types of `tagma-core` plus `base11172`.

| Module | Types |
|--------|-------|
| `tagma-core` | `Coord`, `CoordPath`, `CoordSet`, `CoordSpace<V>`, `CoordCube` |
| `base11172` | `Base11172` serialization |

Heap-backed core types (`CoordSpaceN`, `CoordSetN`, `DynCoordSpace`) and the
modules `tagma-geo`, `tagma-map`, `tagma-sec` follow in later milestones.

## Layout

```
sw/java
├── pom.xml                  aggregator (Java 21, JUnit 5, -Xlint:all)
├── run.sh                   module entry point: mvn verify
├── tagma-core/              artifact org.ssccs.syntagma:tagma-core
└── base11172/               artifact org.ssccs.syntagma:base11172
```

The root `run.sh` runs the Java core through `check_java` whenever `mvn` and
`java` are available, and the CI workflow `java` job builds and tests
`sw/java` with JDK 21 (temurin).

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
| `std::optional<T>` returns | `Optional<T>` / `OptionalInt` | Mirror of optional presence semantics |

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
cd sw/java && ./run.sh  # or: mvn -f sw/java/pom.xml verify
```

Coverage of the translated suites: `CoordTest` (from `test_coord.cpp`),
`CoordPathTest` / `CoordSetTest` / `CoordSpaceTest` (from
`test_core_types.cpp`), `CoordCubeTest` (from `test_coord_cube.cpp`), and
`Base11172Test` (from `test_base11172.cpp`). Rust-only behaviors that the C++
port does not expose, such as the `FlatEntry::and_modify` chain, the
`CoordSpace` equality contract, and `FromIterator` construction, are
documented as follow-ups rather than invented API surface.

## Reference

- C++ port: `sw/cpp` (`tagma_core`, `base11172`)
- Rust reference: `sw/rust` (`core`, `base11172`)
