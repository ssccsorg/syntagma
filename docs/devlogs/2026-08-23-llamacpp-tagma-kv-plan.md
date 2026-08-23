# llama.cpp Tagma KV Investigation Plan

Date: 2026-08-23

Applies the tagma coordinate principle to the llama.cpp KV cache on the SSCCS fork (`ssccsorg/llama.cpp`). The vLLM track (syntagma issue 52) is the CUDA-path venue whose GPU serve benchmark is blocked on NVIDIA hardware. This track targets a GPU-free proof surface: `llama-bench` runs on CPU and Apple Silicon without an accelerator. The document records the verified structure, corrects the factual claims in the initial analysis, and sets the execution phases with decision gates.

## Strategic framing

Analytic conclusion: llama.cpp is a famous open-source LLM engine with a broad individual-developer and edge-device community, and its reference benchmark runs without a GPU. The venue choice removes the hardware blocker that the vLLM track carries. The llama.cpp claim is a venue choice, not a performance prediction; the measured numbers decide the outcome.

## The initial analysis and its corrections

The initial analysis framed llama.cpp as having a legacy unified contiguous `[n_ctx x n_seq_max]` cache and an in-progress paged KV cache modeled on vLLM PagedAttention with a per-sequence block table, citing a 25 to 26 versus 247 concurrent-sequence gain and a block-table lookup cost of 30 percent of decode iteration. Code verification against the fork at master `b0539c43e` corrects the following:

| Claim | Verified state |
| :--- | :--- |
| A paged KV cache is being introduced | No paged cache exists in the current source; the term does not appear in `src/` or `examples/` |
| A per-sequence block table | The indirection is per-token index vectors (`slot_info.idxs`) mapping each token to a cache cell, not a block table |
| A legacy unified `[n_ctx x n_seq_max]` buffer | The cache allocates fixed `[n_embd, kv_size, n_stream]` tensors with `n_stream = 1` (unified) or `n_seq_max`; allocation is cell-based with a ring-buffer search (`v_heads`) |
| 25 to 26 versus 247 concurrent sequences | Unverifiable figure; not found in the codebase or the checked fork history |
| Block-table lookup at 30 percent of decode iteration | Unverified figure originating from third-party vLLM studies, explicitly marked unverified in the vLLM report; it must not be repeated as a llama.cpp measurement |

The verified structural parallel stands on its own: llama.cpp resolves every token through a cell index vector, the same per-position indirection that the tagma write path removes in vLLM. The corrections narrow the claim; they do not remove the opportunity.

## Verified structure

- `llama_memory_i` interface (`src/llama-memory.h`) with multiple implementations (`src/llama-kv-cache.cpp` base, msa, iswa, dsa, dsv4 variants). A new implementation can follow the existing pattern.
- `slot_info.idxs`: per-token cell indices passed into the attention graph (`build_input_k_idxs`, `set_input_k_idxs`), the per-position indirection.
- `find_slot(ubatch, cont)`: a contiguous-slot mode exists, but the normal path calls `find_slot(ubatch, false)`; contiguity is not the default.
- `is_contiguous()` on `slot_info`: the entry point for range arithmetic.
- Cells carry position, shift, and sequence metadata and can be shared across sequences; the ring search starts at `v_heads`.

## Pattern trace from the vLLM integration

The vLLM track (syntagma issue 52) provides reference material, not a template. The two header-only engine files (`kv_coord_map.h`, `kv_allocator.h`) carry over, and the binding, manager, and kernel layers map to llama.cpp equivalents only where the structures align. The value analysis in the following section is specific to llama.cpp and overrides the mechanical mapping where the projects differ.

| vLLM integration component | File | llama.cpp mapping |
| :--- | :--- | :--- |
| Coordinate arithmetic | `csrc/tagma/kv_coord_map.h` | Portable header; the cell offset closed form inside the new memory implementation |
| Range allocator | `csrc/tagma/kv_allocator.h` | Portable header; the allocation core of the new memory implementation |
| Host extension | `csrc/tagma/tagma_bindings.cpp` | Not needed: llama.cpp is pure C++ with no Python binding layer |
| Manager contract | `vllm/v1/core/tagma_kv_cache_manager.py` | The `llama_memory_i` interface; a new implementation follows the msa, iswa, dsa, dsv4 pattern |
| Write path | `BlockTables` tagma mode and `_compute_slot_mappings_tagma_kernel` | `slot_info` construction in `prepare()`; contiguous spans make `idxs` a closed form and the `cpy_k` and `cpy_v` scatter a dense copy, while `get_k` and `get_v` keep their existing contiguous views |
| Fail-closed boundary | Manager rejections of unsupported configurations | Fall back to the base cell mapping when contiguity cannot be maintained |
| Benchmark pattern | `bench_kv_engine.cpp` and `run-vllm-tagma-benchmark.sh` | `llama-bench` as the single bench code point and `run-llamacpp-tagma-benchmark.sh` as the single collection point |

The porting order follows the vLLM phases minus the binding layer: the header-only engine first, then the memory implementation, then the write-path change, then the measurement.

## llama.cpp-specific value analysis

llama.cpp differs from vLLM in the execution model, the kind of indirection, and the memory model. The tagma value hypotheses follow from those differences, and the vLLM mapping is reference material where the structures align.

### What differs from vLLM

| Dimension | vLLM | llama.cpp |
| :--- | :--- | :--- |
| Execution model | CUDA graphs amortize the per-step host work; kernels are the hot path | The ggml graph is rebuilt every step on the host; graph construction is a per-step cost |
| Indirection kind | A block table gathered per position inside attention kernels | Per-token `idxs` vectors built on the host and passed as graph tensors; `get_k` and `get_v` select views through them |
| Memory model | Discrete VRAM, fragmentation-driven paging | Unified memory, fixed `[n_embd, kv_size, n_stream]` tensors; the cell cache tracks usage inside the fixed buffer |
| Contiguity | Not a mode; the block table is the only path | `find_slot(cont=true)` and `is_contiguous()` exist; the unified single-stream mode is already contiguous |
| Sharing | Range-level refcounts | Cell-level sharing across sequences (sequence bitset per cell), transposed V layout, flash-attention block reads |

### Value hypotheses

- H1, host-side graph and bookkeeping cost: the cell scan in `find_slot`, the snapshot and restore in `prepare` (`cells.cp` and `cells.set`), the per-token metadata updates in `apply_ubatch`, the `set_input_*_idxs` fills, and the per-row scatter in `cpy_k` and `cpy_v` are host-side costs that contiguous spans remove. The read path is already range-based: `get_k` and `get_v` return contiguous per-stream views and the attention mask selects the cells. Measurable on the host without a GPU.
- H2, CPU memory locality: contiguous per-sequence KV spans improve cache and TLB behavior in long-context decode, the CPU counterpart of the GPU L2 and TLB argument. This is the llama.cpp-specific form of the structural claim.
- H3, parallel streams: the multi-stream mode interleaves sequences; keeping each sequence contiguous within its stream is the allocation problem that the `find_slot(cont=true)` success rate measures. When contiguity fails, the base cell mapping takes over, the fail-closed boundary.

### Baseline signal interpretation

The Phase A baseline on Apple Silicon (CPU-only) shows prompt processing at 648 tok/s for a 4K prompt and 166 tok/s for a 32K prompt, and decode at 157 and 134 tok/s. The prefill drop is dominated by the quadratic attention cost, so it is by itself a tagma signal. The decode drop is mild; the indirection share is not yet isolated. The Phase A profile must separate the host-side `idxs` construction and the K and V access behavior from the attention arithmetic, which is the actual gate.

### Phase A: local build and profile

Build llama.cpp on Apple Silicon, run `llama-bench` with long-context decode (32K and 128K), and profile the KV access path: the share of decode time spent on cell index construction and lookup, and the memory locality of the K and V access. Instruments on macOS or `perf` on Linux provides the profile.

Gate: the KV access path must be a measurable share of decode time at long context. If the profile shows noise, the track records a measured negative result and stops before implementation.

### Phase B: contiguity measurement

Measure the `find_slot(cont=true)` success rate under parallel streams and fragmentation, quantify the fallback behavior when contiguity fails, and measure the per-token `idxs` construction cost. This decides whether a range-addressed implementation has a stable entry point.

### Phase C: implementation

Follow the pattern trace above. Copy the header-only engine (`kv_coord_map.h`, `kv_allocator.h`) into the llama.cpp fork, add a new `llama_memory_i` implementation (or a contiguity-default mode in `llama_kv_cache`) that keeps per-sequence contiguous cell spans and derives cell indices by arithmetic, so the write path becomes a dense copy of a contiguous destination range and the bookkeeping collapses to range endpoints. The read path needs no change because `get_k` and `get_v` already return contiguous per-stream views. The change must keep the CPU, Metal, and CUDA paths correct; when contiguity cannot be maintained, the implementation falls back to the base cell mapping.

### Phase D: benchmark and contribution decision

Run `llama-bench` before and after on CPU and Apple Silicon, and compare the memory footprint. If the measured improvement holds, decide between an upstream pull request (per the llama.cpp contribution policy, with a human review of every changed line) and a public SSCCS fork. Parity records a measured negative result and closes the track.

## Measured outcome: Phase A and Phase B

Executed on 2026-08-23 on the CPU-only build (Apple M1 Max, Qwen2.5-0.5B Q4_K_M, 32K context) with the host-path microbenchmark in the fork (`pocs/tagma-kv-bench`), which drives the real `llama_kv_cache` functions with synthetic ubatches and times them without attention arithmetic.

| Measurement | Value |
| :--- | ---: |
| T2 host KV bookkeeping per decode step, clean ring | 2.48 us (0.033 % of the 7.47 ms decode step) |
| T2 host KV bookkeeping per decode step, fragmented | 19.7 us (0.26 %) |
| `find_slot(cont=true)` success rate under eviction | 100 % (run of 32 exists), found at 193 us vs 0.94 us base scan |
| Per-sequence append adjacency in unified mode | 0.0 % (sequences interleave) |
| T1 scatter vs dense copy, decode (1 token) | equal at ~6.5 us per op (dominated by per-op dispatch) |
| T1 scatter vs dense copy, V transposed, 256-token chunk | 80.6 us vs 7.95 us (prefill only, ~0.13 % of prefill wall time) |

Analytic conclusion: the host-side KV bookkeeping is three orders of magnitude below the decode step time on CPU, so the Phase A gate as written (a measurable share of decode time) returns a negative result for the host-side cost hypothesis. The read path reads the full ring sequentially through the contiguous per-stream view, so per-sequence contiguity does not change the CPU read pattern. The remaining unmeasured items are the in-graph marginal cost of the scatter nodes inside a real decode graph, the SWA variant (needs a SWA model), and the GPU-side behavior, which stays on the vLLM CUDA track.

## Decision rules

- The GPU-free measurement is the gate; every phase runs on CPU or Apple Silicon.
- Unverified figures (the 30 percent figure and the sequence-count gains) are excluded from claims until measured on this fork.
- The vLLM track stays the CUDA-path venue; this track is the GPU-free venue.

## References

- llama.cpp fork, ssccsorg/llama.cpp, master `b0539c43e`
- vLLM tagma GPU plan, 2026-08-23-vllm-tagma-gpu-plan.md in this directory
- vLLM KV Cache report, ssccs docs, works/llms/vllm/kvcache
- TagmaVec assessment, 2026-08-22-tagmavec-turbovec-assessment.md
