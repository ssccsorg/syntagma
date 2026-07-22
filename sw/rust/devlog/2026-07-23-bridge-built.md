# The bridge is built: what the benchmarks told us

After the benchmark corrections and the conversion cost analysis, the tagma-kv picture is finally consistent across all three scales. The results confirm the strategy but also reveal where the real engineering challenges lie.

## What the corrections changed

The previous CoordKVN<2> benchmark was optimistic. It used only 1,296 unique keys at 10k scale — a dataset that fits entirely in L1 cache. The corrected version uses the full 65,536 2-byte key space via the `by_coordkey` API, matching the same key count as CoordKV2. The result: CoordKVN<2> at 10k went from 21.6 ns to 22.7 ns, a 5% increase that reflects the actual memory hierarchy cost of a larger working set.

CoordKV2 was unaffected because it already used the full 65k key space. HashMap showed 21.9 ns at 10k (down from 23.8 ns in previous runs, within measurement noise). The structural conclusions — CoordKV2 is scale-invariant, CoordKVN and CoordKV2 are functionally equivalent at N=2, HashMap costs rise with cache pressure — all hold with slightly different numbers.

## The conversion cost breakdown

The most important analytical contribution is the three-layer decomposition of CoordKV2 get latency:

| Layer | Cost | Characterization |
|-------|------|------------------|
| str-to-CoordKey | 5 ns | one-time, amortizable |
| Box-to-Vec clone | 16.67 ns | API artifact, eliminable |
| Slot load | 0.39 ns | physical limit |

The 5 ns conversion is the only cost that belongs to the bridge. It is paid once at the entry boundary and skipped by `by_coordkey` API. The 16.67 ns Vec clone is an artifact of the current API returning owned values to match HashMap's `insert`/`remove` contract. A reference-returning variant would eliminate it, dropping CoordKV2 get from 22 ns to 5 ns (conversion only). The slot load at 0.39 ns is the pure Tagma promise — array index at gate delay.

HashMap has no equivalent decomposition. SipHash, bucket probe, and collision resolution are inseparable — every get pays the full 23.79 ns.

## The one-time cost chart

The cumulative cost chart (`fig-bench-kv-onetime-cost.qmd`) visualizes the strategic difference. CoordKV2 via `by_coordkey` pays ~22 ns once and then ~1 ns per operation. HashMap pays ~24 ns per operation forever. At 100,000 operations, the gap is 23.8x. At 10M operations, it is still 23.8x. This is not a performance tuning detail — it is a structural consequence of the addressing model.

## Where the real engineering is

The benchmarks confirmed that the on-memory dense (CoordKV2) and sparse (CoordKVN) variants work as designed. The remaining challenges are at the mmap and disk layers:

1. **Page fault cost per lookup**: CoordSpaceM with N=3 maps 1.27 TB of virtual address space. A single CoordPath lookup touches 3 Coord slots, potentially on 3 different pages. If those pages are not resident, 3 page faults at ~10 µs each turn a 22 ns lookup into 30 µs. The density of actual entries relative to the full address space determines how often this happens.

2. **TLB coverage**: With 4 KB pages, 1.27 TB requires 332M page table entries. Even with 2 MB huge pages, the working set may span thousands of entries. If the access pattern is random (hash-like), TLB miss rate dominates. If the access pattern exploits spatial locality (prefix scan, range query), TLB pressure drops.

3. **No index tree**: Unlike B-tree or LSM-tree, Tagma on disk has no index structure to navigate. The coordinate IS the address. The question is how to map a 16-bit Coord (0..11171) to a disk block efficiently when the block size is 4 KB. With 11172 slots per level, each level spans ~44 KB in dense mode — approximately 11 blocks. A lookup at N=19 would touch 19 Coords × 11 blocks each = 209 random block reads worst case. This is worse than B-tree. The solution is to increase slot density per block by packing multiple Coords into one block, or to fall back to the sparse tree (CoordSpaceN) for disk.

## What we know for certain

- CoordKV2 (119 MB, N=2) is the practical deployment target. Its performance is scale-invariant and its slot access is at the physical limit.
- CoordKVN provides the same interface for sparse or memory-constrained deployments.
- The bridge cost (str-to-CoordKey conversion) is below the SipHash baseline, confirmed across three scales.
- The mmap and disk layers are unsolved engineering problems, not theoretical limitations. The addressing model is sound; the page table is not.
