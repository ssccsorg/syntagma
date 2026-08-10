# Tagma Core: Rust implementation postmortem

Concepts, language constraints, hardware limits, and the full issue registry after the 4-n-character-coordmap refactor (PR #5).

---

## 1. Tagma concept vs Rust language limitations

### 1.1 Const generic N has no type-level bounds

Tagma's multi-character addressing requires a compile-time depth parameter N. Rust provides `const N: usize` generics but no way to express bounds like `N > 0` or `N <= 19` at the type level.

```
CoordSpaceN<0, V>    → compiles, panics at runtime (assert! in new())
CoordSpaceN<65535, V> → compiles, OOM at runtime
```

**Workaround:** `assert!(N > 0)` in `new()`. For the upper bound, engineering judgment: any N above 19 has no Tagma use case, but blocking it in the type system is not possible without const-expression evaluation in trait bounds, which Rust does not yet support.

### 1.2 No const trait specialization

`CoordSpaceN<1, V>` and `CoordSpaceN<N, V>` (general) are separate `impl` blocks. They cannot share methods: Rust does not allow one `impl` block to override a method from another. This forces:

- `CoordSpaceN<1, V>::iter_flat()` — efficient inline scan, yields `(Coord, &V)`
- `CoordSpaceN<N, V>::iter_tree()` — Vec-collecting tree walk, yields `(CoordPath<N>, &V)`
- A single `iter()` name cannot satisfy both without ambiguity (multiple applicable items error).

The same duplication applies to `Entry`, `Drain`, `FromIterator`, `IntoIterator`, `Index` — all must be implemented separately for N=1.

### 1.3 `core::mem::zeroed()` for large fixed arrays

`CoordFlatMap` allocates `[Option<V>; 11172]` as a stack inline array. Safe initialization requires 11,172 writes. The `zeroed()` trick avoids this:

```rust
let slots = unsafe { core::mem::zeroed() };  // flat.rs L37
```

This assumes `Option<V>` has an all-zero bit pattern for `None`. This is the de facto ABI across all Rust versions and targets, but the language specification does not formally guarantee it.

**If this breaks in a future compiler** (extremely unlikely), the fix is a safe initialization loop:

```rust
let mut slots: [core::mem::MaybeUninit<Option<V>>; 11172] =
    unsafe { core::mem::MaybeUninit::uninit().assume_init() };
for s in &mut slots {
    s.write(None);
}
let slots = unsafe { core::mem::transmute::<_, [Option<V>; 11172]>(slots) };
```

Cost: construction latency increases from ~1 ns to ~3-5 µs (11,172 stores).

### 1.4 No `const fn` on trait methods

Tagma's coordinate arithmetic (decomposition into 19 x 21 x 28 axes, validation, composition) is pure arithmetic that could theoretically be `const fn`. But `const trait` and `const fn` in traits are still nightly-only in Rust 2024 (stabilizing in later editions). This prevents compile-time coordinate computation in generic contexts.

Impact: Minor. The arithmetic is fast enough at runtime (single-digit nanosecond range).

### 1.5 Box<[T]> vs [T; N] on the stack

`CoordSpace` uses `Box<[Option<V>; 11172]>` (heap allocation per node) while `CoordFlatMap` uses `[Option<V>; 11172]` on the stack. The tradeoff:

| Property | CoordFlatMap | CoordSpaceN<1> |
|----------|-------------|-----------------|
| Allocation | Zero (inline) | Heap (22+ KB) |
| Construction | ~1 ns (zeroed) | ~3-5 µs (vec collect) |
| Access | Single deref | Single deref (box) |
| Move cost | 22 KB memcpy | 8 byte pointer copy |

CoordSpaceN<1> exists only because the const generic `CoordSpaceN<N>` must handle N=1 as a valid instantiation. Users should use `CoordSpace` (= `CoordFlatMap`) for the zero-alloc single-character case.

---

## 2. Hardware constraints

### 2.1 Per-level memory cost

Each level is a fixed 11,172-slot array. With lazy allocation (CoordSpace), only paths that are actually written consume memory. Memory per node:

| Value type | Leaf node | Branch node |
|-----------|-----------|-------------|
| `()` | 22 KB | 89 KB |
| `u32` | 44 KB | 89 KB |
| `u64` | 89 KB | 89 KB |
| `Box<dyn Trait>` | 89 KB | 89 KB |

For CoordSpaceN6, a single leaf path at the deepest level creates one Branch at level 0 + one Branch at level 1 + ... + one Leaf at level 5 = 5 x 89 KB + 44 KB = 489 KB for a single `u32` entry. This is the worst-case memory overhead for sparse trees.

### 2.2 Cache behavior

- **N=1 (CoordFlatMap/CoordSpaceN<1>):** Single 22-89 KB array. A `get()` is one load from a structure that typically fits in L1 cache (32 KB) for small V, or L2 (512 KB-2 MB) for larger V.
- **N=2:** Two array accesses. First array may be L1, second may be L1 if the first coord is reused frequently.
- **N=6:** Six pointer chases through 89 KB Branch arrays. Each level is a separate heap allocation. Cache misses are likely for sparse access patterns but rare for sequential access to the same prefix branch.
- **N=19:** 19 pointer chases. Worst-case: 19 cache misses (~150 ns each at 10 ns DRAM access = 2.85 µs per get). Compare to HashMap: 2-3 cache misses per get (~300-450 ns). At N=19, Tagma loses to HashMap on pure cache miss cost.

**Mitigation:** N=19 exists only for SHA-256-scale identity spaces where the number of entries is typically very small (1-100). In practice, the tree is never fully populated, and the cost is dominated by the 19 dereferences, not memory bandwidth.

### 2.3 Memory fragmentation

CoordSpace allocates one node per unique prefix path. For sparse distribution:

```
CoordSpaceN6 with 1000 entries at random coords:
  Expected unique prefixes at level 0: ~1000 (spread across 11,172 slots)
  → 1000 Branch nodes at level 0
  → 1000 Branch nodes at level 1 (if each prefix maps to unique level-1 coord)
  → ... eventually ~6000 heap allocations for 1000 entries
```

Each allocation is a separate `Box<[Option<...>]>`. The allocator sees many medium-sized (89 KB) blocks. Fragmentation is a practical concern for long-running processes with many insert/remove cycles.

### 2.4 Stack depth for DynCoordSpace recursion

`DynCoordSpace::insert_rec` and `remove_rec` are recursive. Each call adds a stack frame. At path depth 100+ (pathological, not practical), the recursion reaches ~100 frames. Rust's default stack is 2 MB (Linux) or 8 MB (macOS), so this is safe for realistic depths. But a deeply recursive insert could panic on very small stacks (embedded, 4 KB).

**This is acceptable for 0.1.0.** An iterative (loop + explicit stack) version would avoid recursion but adds complexity.

---

## 3. Full issue registry

### 3.1 Resolved issues

| # | Title | Resolution | Commit |
|---|-------|-----------|--------|
| **6** | DynCoordSpace mixed-depth path destroys shallow values | `Slot::Both(V, Box<Child>)` — both value and sub-node coexist | PR #5 |
| **11** | CoordSpace missing Clone/PartialEq/Debug | Manual Debug (occupied count), PartialEq+Eq, Clone derive | PR #5 |
| **13** | DynCoordSpace stress test | 100-path insert/verify/remove/reverify cycle | PR #5 |
| **14** | Benchmark coverage gap | CoordSpaceN2 insert/get 1000 added | PR #5 |
| **15** | iter_flat vs iter_tree naming | N=1: `iter_flat()`. N>1: `iter_tree()`. Distinction documented. | PR #5 |
| | FlatMap missing iter_mut | `FlatIterMut` + `values_mut()` added | PR #5 |
| | clear() heap reallocation | `clear_node()` tree walk instead of re-allocating | PR #5 |

### 3.2 Open issues (accepted for 0.1.0)

| # | Title | Impact | Workaround |
|---|-------|--------|------------|
| **7** | CoordSpaceN<1> duplicates FlatMap | Minor — user confusion | Use `CoordSpace` alias for N=1 |
| **8** | CoordSet word boundary | Analyzed as already safe for 0.1.0 | None needed |
| **9** | Drain slower than HashMap | 27.7 µs vs 19.7 µs for 11k (minor) | Use `clear()` instead of `drain()` |
| **10** | TreeIter Vec collection + double lookup | O(entries) memory for iteration | Use `iter_flat()` for N=1 |
| **16** | Residual risk register (this issue) | Non-blocking concerns | See issue body |

### 3.3 Never-tracked but known limitations

| Limitation | Category | Notes |
|-----------|----------|-------|
| `core::mem::zeroed()` assumption | Rust soundness | Accepted as de facto standard |
| `unreachable!()` in Node dispatch | Maintenance risk | Guarded by type invariants |
| DynCoordSpace `&[Coord]` vs CoordSpace `CoordPath<N>` | API consistency | Accepted as natural Rust idiom |
| CoordSpace N=65535 (no upper bound) | Engineering domain | OS memory protects |
| No `const fn` for Coord arithmetic | Rust nightly limitation | Not needed at runtime |

---

## 4. Architecture decisions reaffirmed

1. **Coord is a struct, not a trait.** Trait generalization would add complexity with no current use case. All map types use Coord directly as an index.

2. **CoordPath is an index path, not a key.** CoordPath has no `Hash` bound. Each Coord is a direct array index, not a hash input. This is the fundamental Tagma principle.

3. **No_alloc is a feature gate, not a separate crate.** The `alloc` feature (default: on) gates heap-backed types. `Coord`, `CoordPath`, `CoordSet`, `CoordFlatMap` are always available without alloc.

4. **HashMap compatibility is API-level, not trait-level.** CoordSpace series exposes the same method names and signatures as HashMap (`get`, `insert`, `remove`, `entry`, `iter`, `clear`, etc.) but does not implement the `HashMap` trait (there is no such trait in std).

5. **Exhaustive validation over 11,172 coordinates.** Every Coord is valid by construction (`Coord::new` returns `None` for out-of-range). The CoordSpace slot array covers exactly 11,172 entries — no hash collisions, no load factor, no resize.

---

## 5. What to fix before 1.0

Priority order:

1. **#10 — TreeIter streaming** — Replace Vec collection with explicit-stack DFS. Removes O(entries) memory allocation from `iter_tree()`.

2. **DynCoordSpace iter()** — Currently DynCoordSpace has no iteration support. Add `iter()` that walks the dynamic-depth tree.

3. **`zeroed()` safe replacement** — Replace with documented `MaybeUninit` + transmute pattern, or accept the 3-5 µs construction cost.

4. **CoordSpaceN6/12/19 full benchmark suite** — Current benches only cover CoordFlatMap and CoordSpaceN2. Add CoordSpaceN6/12/19 benchmarks for insert/get/iter across varying entry counts (sparse to dense).

5. **Miri test for unsafe blocks** — Run `cargo miri test` on the `zeroed()` path and the `IterMut` raw pointer path to verify no UB.

---

*Last updated: 2026-07-16*
