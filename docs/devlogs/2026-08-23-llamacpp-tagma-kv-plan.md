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

## Execution phases

### Phase A: local build and profile

Build llama.cpp on Apple Silicon, run `llama-bench` with long-context decode (32K and 128K), and profile the KV access path: the share of decode time spent on cell index construction and lookup, and the memory locality of the K and V access. Instruments on macOS or `perf` on Linux provides the profile.

Gate: the KV access path must be a measurable share of decode time at long context. If the profile shows noise, the track records a measured negative result and stops before implementation.

### Phase B: contiguity measurement

Measure the `find_slot(cont=true)` success rate under parallel streams and fragmentation, quantify the fallback behavior when contiguity fails, and measure the per-token `idxs` construction cost. This decides whether a range-addressed implementation has a stable entry point.

### Phase C: implementation

Follow the `llama_memory_i` pattern: a range-addressed implementation, or a contiguity-default mode, that keeps per-sequence contiguous cell spans and derives cell indices by arithmetic. Port the KVAllocator spec from the vLLM track (contiguous ranges, coalescing, refcounted sharing, LRU eviction) to the llama.cpp memory model. The change must keep the CPU, Metal, and CUDA paths correct.

### Phase D: benchmark and contribution decision

Run `llama-bench` before and after on CPU and Apple Silicon, and compare the memory footprint. If the measured improvement holds, decide between an upstream pull request (per the llama.cpp contribution policy, with a human review of every changed line) and a public SSCCS fork. Parity records a measured negative result and closes the track.

## Decision rules

- The GPU-free measurement is the gate; every phase runs on CPU or Apple Silicon.
- Unverified figures (the 30 percent figure and the sequence-count gains) are excluded from claims until measured on this fork.
- The vLLM track stays the CUDA-path venue; this track is the GPU-free venue.

## References

- llama.cpp fork, ssccsorg/llama.cpp, master `b0539c43e`
- vLLM tagma GPU plan, 2026-08-23-vllm-tagma-gpu-plan.md in this directory
- vLLM KV Cache report, ssccs docs, works/llms/vllm/kvcache
- TagmaVec assessment, 2026-08-22-tagmavec-turbovec-assessment.md
