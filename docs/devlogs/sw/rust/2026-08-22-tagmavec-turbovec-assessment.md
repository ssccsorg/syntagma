# TagmaVec assessment: what the TurboVec codebase shows and what must be measured

The vector-search extension proposal, TagmaVec, was discussed as "TurboVec
without hashing": keep the quantization engine, replace the `id_map` hash
table with the tagma coordinate space, and claim index-free deterministic
search. This devlog grounds that discussion in the actual codebases, the
local `turbovec` checkout and the syntagma primitives, separates what the
code supports from what it contradicts, and lists the measurement-first
steps before any claim.

## The verified shape of TurboVec

`turbovec` 1.0.0 is a Rust library. The engine is a positional index,
`TurboQuantIndex`, with a quantization pipeline and a SIMD search kernel:

| File | Lines | Role |
| :--- | ---: | :--- |
| `search.rs` | 4,659 | SIMD scoring pipeline, the hot path |
| `lib.rs` | 4,620 | `TurboQuantIndex`, add and search API |
| `io_v7.rs` | 2,078 | serialization |
| `encode.rs` | 1,933 | quantization encoding |
| `id_map.rs` | 1,638 | `IdMapIndex`: the external-ID hash layer |
| `pack.rs` | 1,425 | bit packing |

The search pipeline scores queries against quantized vectors with
nibble-split lookup tables and architecture-specific SIMD kernels, NEON on
ARM and AVX-512BW with an AVX2 fallback on x86, selected at runtime. Single
queries above a block-count threshold run in a rayon pool, with measured
A/B interleaving documented in the source comments. The Rust 1.89 floor is
set by the AVX-512 intrinsics, not by design choice.

`IdMapIndex` wraps the positional index with a bidirectional `u64` to slot
`HashMap`. The wrapper's own doc comment states the search cost: the inner
search plus an O(nq times k) ID translation pass over the returned slot
indices. The hash layer is the only hash table in the path, and it is a
post-processing pass over search results.

## What the code confirms and what it contradicts

The proposal's factual claims check out. The hash table exists, it is a
`HashMap` over external IDs, and it sits between the caller and the
positional index. The proposal's performance logic does not follow.

The search bottleneck is not the ID translation. `search.rs` is the largest
file and the one with SIMD kernels and block-parallel dispatch; the score
scan over the quantized codebook is the engine. The ID translation is an
O(nq times k) pass over the results the scan already produced. Removing the
hash table removes that pass, and nothing else. The codebase itself shows
the authors already minimized the hash cost: the custom `IdHasher`
(multiply-shift plus a two-round splitmix finalizer) exists precisely
because the map is in the remove and update path, and its comment cites a
measured 476 ms to 0.2 ms improvement on adversarial ID shapes.

The deterministic O(1) claim does not transfer. Similarity search is not
key addressing. A query vector has no precomputable address for "the most
similar stored vector"; the metric that decides the value of an ANN index
is recall at a given query rate, measured on ANN-Benchmarks, not
addressability.

## The structural mismatch: code lattices versus tagma lattices

Quantization is itself a coordinate mapping: product quantization splits a
vector into subvectors and assigns each a codebook index, so a vector
becomes a tuple of indices, a coordinate in a discrete code lattice. This
is where a tagma reading of the problem would naturally start.

The lattice is the obstacle. A code lattice with a codebook of C entries
per subvector and k subvectors has C to the k points. TurboVec packs 2 to 4
bits per subvector, so the lattice is astronomically large and sparsely
populated. The tagma dense-addressing primitives are built for bounded
enumerable lattices: the syntagma core defines `N_VALID` as 11,172, the
19 by 21 by 28 base11172 space, and the measured spatial queries, CoordCube
proximity at 285 ns (radius 1) and 626 ns (radius 2) on 10K dense entries,
walk a neighborhood of that bounded space. A high-dimensional code lattice
does not fit that model, and the transfer is an open question, not a given.

## What to analyze first

Measurement comes before design, in this order:

1. Decompose the TurboVec search cost at scale: the SIMD score pass, the
   ID translation pass, and I/O. The question is the share of the ID
   translation at realistic sizes, 10M vectors, nq of 1 and 100, k of 10
   and 100. If the share is below one percent, replacing the hash is a
   micro-optimization, not a structural claim.
2. Measure the code lattice. The bit width and subvector count define C to
   the k; the question is whether any bounded sub-lattice exists that
   preserves acceptable recall. Recall is the constraint, and it must be
   measured before any speed claim.
3. Run ANN-Benchmarks as the decision target: recall at k against queries
   per second, tagma candidate generation against the TurboVec linear
   scan.

## What to investigate in the codebases

- `turbovec`: `search.rs` hot loop and the block-parallel gate, `encode.rs`
  rotation, calibration, and Lloyd-Max steps, `id_map.rs` remove path and
  hasher, `pack.rs` bit layout, `io_v7.rs` load path
- `syntagma`: the spatial primitives in `sw/rust` (CoordCube proximity and
  proximity hamming) and `sw/cpp`, whether the neighborhood walk
  generalizes from the 2D and 3D bounded lattices to a code lattice, and
  what a code lattice bounded to the 11,172 space would cost in recall
- The vLLM KV cache dtype layer is a boundary, not a path: quantization
  compresses, coordinate addressing addresses, and the two are orthogonal

## The honest hypotheses

Three candidate shapes remain, in increasing ambition:

- H1, ID mapping: replace the `u64` to slot `HashMap` with coordinate
  addressing. Real and bounded, but its value is exactly the ID
  translation share measured in step one.
- H2, candidate generation: embed vectors into a bounded code lattice and
  walk the neighborhood with tagma spatial queries instead of scanning
  every code. This is the structural claim, and it is blocked by the
  lattice size and recall questions.
- H3, exact addressing: address a known, bounded vector set whose IDs are
  coordinates. This is the direct transfer of the CERN and vLLM pattern,
  and its scope is narrow.

## The decision rule

The third verification case is claimed only after the measurement: if the
ID translation is a single-digit share of search time, H1 is an
optimization with a bounded result; H2 needs a bounded code lattice with
measured recall before a benchmark exists; H3 stands alone as a narrow
exact-addressing case. Until one of these is measured on ANN-Benchmarks,
TagmaVec is a proposal with a codebase-grounded analysis, not a result.

## Next

The concrete next action is the search-cost decomposition from the
analysis-first list, run against the local `turbovec` checkout. This track
is sequenced after the vLLM KV cache GPU measurement: the two tracks share
the syntagma core but no code, and the same rule applies, no claim ahead of
measurement.
