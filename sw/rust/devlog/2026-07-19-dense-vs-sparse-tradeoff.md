# Dense vs Sparse: when Tagma cannot fit in one memory

Tagma promises collision-free O(1) direct addressing. The promise holds for any N: a
CoordPath of N syllables addresses exactly one slot in the $11{,}172^N$ space, no hashing,
no collisions. But the promise has a material prerequisite: the address space must
actually exist in memory.

## The constraint

Contiguous memory cost for the full address space:

| N | Address space $11{,}172^N$ | Contiguous memory ($\text{Option<()>}$) | Feasible? |
|---|---------------------------|----------------------------------------|-----------|
| 1 | $1.12 \times 10^4$ | 11 KB | Yes |
| 2 | $1.25 \times 10^8$ | 119 MB | Marginal |
| 3 | $1.39 \times 10^{12}$ | 1.27 TB | No |
| 6 | $1.94 \times 10^{24}$ | $1.94 \times 10^{12}$ TB | No |
| 19 | $\approx 2^{256}$ | Unmeasurable | No |

At N=2 the full space is physically allocable (119 MB, V=()) but wasteful for sparse workloads.
At N=3 (1.27 TB) it is no longer possible with current single-node hardware.

This is not a Tagma limitation. It is a physics limitation: $11{,}172^N$ grows as
$O(10^{4N})$ while physical memory grows as $O(2^{\text{address bits}})$. No addressing
scheme, hash-based or otherwise, can allocate $10^{12}$ entries on a machine that has
$10^{10}$ bytes. The difference is that hash-based schemes hide this behind probabilistic
compression, while Tagma exposes it as a structural choice.

## The two implementations

### Dense: CoordSpace / CoordSet (N=1)

Preallocates every slot. Zero heap allocation after construction. Single-cycle decode.

```rust
// CoordSpace: 22-89 KB inline array
pub struct CoordSpace<V>([Option<V>; 11172]);

// CoordSet: 175 u64 bit array (1.4 KB)
pub struct CoordSet([u64; 175]);
```

Every Coord resolves in one array load or bitwise operation. No branches, no hashing,
no heap indirection. This is the complete Tagma promise.

Construction uses `unsafe { core::mem::zeroed() }` relying on `Option`'s niche
optimization producing an all-zero `None` pattern. This is the de facto standard
across the Rust ecosystem but is not formally guaranteed by the language spec (see
postmortem `2026-07-16-tagma-core-rust-hardware-postmortem.md` section 1.3).

### Sparse: CoordSpaceN\<N\> (N > 1)

Lazy tree. Root is always allocated; deeper nodes created only for written paths.

```rust
pub struct CoordSpaceN<const N: usize, V> {
    root: Node<V>,  // Leaf if N=1, Branch if N>1
    len: usize,
}

enum Node<V> {
    Leaf([Option<V>; 11172]),
    Branch([Option<Box<Node<V>>>; 11172]),
}
```

Lookup cost for path $[c_0, c_1, ..., c_{N-1}]$:

1. `root.as_branch()[c_0]` -- first array load
2. Dereference Box -- first heap access
3. `child.as_branch()[c_1]` -- second array load
4. Dereference Box -- second heap access
5. ...
6. `leaf.as_leaf()[c_{N-1}]` -- final array load

Total: N array loads + N-1 Box dereferences + N-1 enum matches.

No hashing at any level. The same address always reaches the same slot. This is
collision-free by construction -- the tree is not a hash table with buckets; it is
a structural decomposition of the address into N direct array indices.

### CoordSetN (N > 1)

A zero-allocation set view over CoordSpaceN, storing `()` as the value. Set
operations are tree walks:

| Operation | Cost | Notes |
|-----------|------|-------|
| `contains` | O(N) tree traversal | Same as CoordSpaceN lookup |
| `insert` / `remove` | O(N) tree traversal | Delegates to CoordSpaceN |
| `is_subset` | O(min(len) $\times$ N) with early exit | Returns false on first mismatch |
| `is_disjoint` | O(min(len) $\times$ N) | Returns false on first overlap |
| `union` | O((len(a) + len(b)) $\times$ N) | Iterates both, no dedup check needed |
| `intersection` | O(min(len) $\times$ N) | Iterates smaller, checks in larger |
| `difference` | O(len(self) $\times$ N) | Iterates self, checks in other |

Compare with CoordSet (N=1):

| Operation | CoordSet (N=1) | CoordSetN (N>1) |
|-----------|----------------|-----------------|
| `contains` | 1 bit op | N dereferences |
| `insert` / `remove` | 1 bit op | N array accesses + alloc |
| `union` | 175 word ops | O(len(a) + len(b)) tree walk |
| `intersection`, `difference` | 175 word ops | O(min) tree walk |
| `is_subset`, `is_disjoint` | 175 word ops | O(min) tree walk |

CoordSetN exists for API consistency: when your data is already in a CoordSpaceN,
adding a parallel CoordSet for set operations costs only the `()` storage per entry.
It is not competitive with HashSet for bulk operations.

## Memory cost per node

Each allocated node is a fixed 11,172-slot array, regardless of occupancy. The full
table from the implementation:

| Value type | Leaf node | Branch node |
|-----------|-----------|-------------|
| `()` | 22 KB | 89 KB |
| `u32` | 44 KB | 89 KB |
| `u64` | 89 KB | 89 KB |
| `[u8; 32]` | 379 KB | 89 KB |

A single entry at the deepest level of CoordSpaceN6 requires 5 Branch nodes (prefix
levels 0-4) + 1 Leaf node (level 5) = $5 \times 89 + 44 \approx 489$ KB for a single
`u32`. This is the worst-case memory overhead of lazy tree allocation.

In practice, entries with shared prefixes share Branch nodes. 1,000 entries distributed
across 10 distinct prefixes at level 0 share the same root Branch and diverge only at
deeper levels. The overhead per entry decreases as density increases.

## Cache behavior

Measured on Apple M1:

| Depth | Access pattern | Typical latency | Bottleneck |
|-------|---------------|-----------------|------------|
| N=1 | Single array load | 0.38 ns | L1 hit (22-89 KB fits in L1) |
| N=2 | 2 loads + 1 Box | 0.87 ns | L1 hit (root 89 KB, leaf may share) |
| N=3 | 3 loads + 2 Box | 2.66 ns | L1-L2 boundary |
| N=6 | 6 loads + 5 Box | 5.60 ns | L2-L3, some DRAM |
| N=12 | 12 loads + 11 Box | 13.2 ns | DRAM dominant |
| N=19 | 19 loads + 18 Box | 53.2 ns | All DRAM, pointer chase |

At N=19, each dereference is a separate heap allocation. CPU prefetch cannot predict
the Box pointer target. Worst case: 19 sequential DRAM reads at ~10 ns each = 190 ns
minimum for the memory subsystem, plus CPU decode = ~53 ns measured (Apple M1 has
fast memory controller and large cache).

Compare with HashMap on the same N=19 lookup: HashMap requires computing SHA-256 of
the key (225 ns), then 2-3 cache misses for bucket traversal (~300-450 ns). Tagma's
53.2 ns is still 4-5x faster despite the worst-case pointer chase, because there is
no hashing cost and no collision resolution.

## The naming convention

The naming tells the reader which category each type belongs to:

| Name | Implementation | Tagma principle |
|------|---------------|-----------------|
| `CoordSpace` | Dense array (inline, 22 KB) | Complete (no alloc) |
| `CoordSet` | Bit array (inline, 1.4 KB) | Complete (no alloc) |
| `CoordSpaceN<N>` | Sparse tree (heap) | Fallback |
| `CoordSetN<N>` | Tree set (heap) | Partial |
| `DynCoordSpace` | Dynamic-depth trie (heap) | Fallback |

The `N` suffix is structural, not cosmetic. It says "this type allocates on the heap
and accesses cost N dereferences." The concrete aliases carry the suffix consistently:
`CoordSpaceN2`, `CoordSpaceN6`, `CoordSpaceN19`.

Before the convention was established, the aliases were `CoordSpaceN2`, `CoordSpaceN6`,
etc., which was misleading: they looked like `CoordSpace` variants with different
depths, but they had fundamentally different implementation characteristics (heap
allocation, tree traversal). The rename was applied retroactively in PR #27.

## The hardware roadmap

The dense-vs-sparse tradeoff exists only because single-node memory is finite.
A cluster of physical nodes can restore the complete Tagma principle at any scale.

Each node hosts a true dense CoordSpace (11,172 slots, zero allocator, single-cycle
decode) for a slice of the address space. A topology mapping function $\phi_k$ assigns
each sub-coordinate to a physical node. The coordinate itself carries its routing:
given $(c_0, c_1, ..., c_{N-1})$ and current recursion depth, any node determines
whether each sub-coordinate is local or remote by applying $\phi_k$.

For SHA-256 scale (N=19), a cluster of 20 nodes, each a true O(1) dense array,
covers the full address space. No tree traversal, no heap allocation, no hash
computation. Each node resolves its local sub-coordinate in a single cycle and
passes the remaining path to the next node. The total latency is O(number of nodes)
at network speed, not O(depth) at memory speed.

This is the long-term direction of the synTagma coordination layer. The software
fallback (CoordSpaceN) exists to make Tagma usable today on single-node hardware.

## Summary

| Category | Implementation | When to use |
|----------|---------------|-------------|
| Complete Tagma | CoordSpace, CoordSet | N=1, any workload. Zero alloc, single-cycle. |
| Software fallback | CoordSpaceN\<N\> | N>1, single node. Tree walk but no hashing. |
| Hardware cluster | CoordSpace $\times$ N nodes | N>1, distributed. Theoretical roadmap. |

The dense-vs-sparse split is not a Tagma limitation -- it is a honest engineering
consequence of finite memory, documented so that users understand exactly what they
get and what they trade at each depth.
