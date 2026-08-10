# The fixed-depth tree binding: why the tree is the right shape

A review of chton's TreeStrategy, the persistent binding that materializes the CoordSpaceN layout over an origin. The question was whether a tree was the best structure for coordinate-addressed storage, and what the structure costs that the implementation does not yet solve.

## What TreeStrategy is

`TreeStrategy<N>` is a fixed-depth radix tree over coordinate digits. Each of the N path coords selects one level; every node is 11,172 slots wide (branch nodes hold child offsets, leaf nodes hold record offsets, and offset 0 is the absent sentinel). Nodes start with an occupancy bitmap (1397 bytes) so enumeration reads one bitmap instead of scanning the fan-out. Records live in a bump area with a free list, the file layout is the storage format with no serialization step, and a self-describing header records depth, node size, and record slot size so a reopen is O(1) and shape mismatches are rejected as corruption.

Node size: 1397 + 11172 x 8 = 90,773 bytes, about 89 KB per node.

## Why a tree and not something else

The alternatives and their failure modes:

| Alternative | Failure against the paradigm |
|-------------|------------------------------|
| Dense array (CoordSpaceM) | O(1) but virtual space explodes (1.27 TB at N=3), TLB pressure, small N only |
| HashMap | unordered, collision resolution, not provable |
| BTreeMap | order and determinism, but an index structure; the paradigm removes indexing |
| B-tree / LSM | one-dimensional key optimization, ignores N-dimensional coordinate structure, 10-50x write amplification |

A fixed-depth trie is structurally right for coordinate addressing: deterministic, order-preserving (a prefix scan is a subtree walk), sparse-friendly (bitmap), and provable. The performance invariant in the module states it directly: the coordinate is the address, resolution is per-level array indexing O(depth), and enumeration is proportional to materialized records, never to the address space.

## The security trajectory

CoordPath is a future security primitive. A radix tree is the natural substrate for authenticated structures: levels are Merkleizable, and existence and non-existence proofs can be built per level. Dense arrays and hash maps do not offer this. Choosing the tree now keeps the binding on the path to verifiable storage without a later structural rewrite.

## Open engineering costs

Two costs are acknowledged, both engineering rather than structural.

Fixed node size. A node is materialized at about 89 KB regardless of occupancy. Sparse deep trees (N=19-21) with random paths can approach one or two records per node, pushing storage cost toward node size per record. The bitmap helps enumeration, not node size. Mitigations are path compression (skip single-child levels), partial node materialization, and packing multiple coords per block, the last already flagged in the bridge-built entry.

Cold random disk access. A lookup hops N nodes, and each hop may touch a bitmap block and an offset block. At N=21 the worst case is tens of random block reads before the page cache warms. The FIH 19-axis store already operates at this scale, so the regime is proven rather than hypothetical.

## Conclusion

The structural choice is sound and the implementation is well engineered: self-describing header, load-time corruption checks, single-writer contract, and a documented separation between the indexing layer and the storage layer. The open items are node compactness and disk locality, which are optimization work, consistent with the policy that performance follows correctness.
