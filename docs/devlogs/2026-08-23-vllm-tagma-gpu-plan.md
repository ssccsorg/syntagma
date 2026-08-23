# vLLM Tagma GPU Plan

Date: 2026-08-23

Status snapshot and execution plan for the GPU measurement phase of the tagma KV cache backend on the SSCCS vLLM fork, tracked under ssccsorg/syntagma issue 52. The plan records the bottleneck candidates identified in the codebase, the execution phases with their decision gates, and the GPU resource path.

## Status snapshot

The integration pull request merged into the SSCCS vLLM fork main branch (commit 86339f249, head fcdeae932). The `vllm.tagma` host extension builds through the real CMake target (SABI 3.11), and the container CI passes three jobs: C++ engine tests under ASan and UBSan, the host benchmark smoke, and the Python software-level tests. The benchmark runner script and the runbook are committed in this directory (`run-vllm-tagma-benchmark.sh`, `2026-08-20-vllm-tagma-benchmark-runbook.md`).

The development machine is Apple Silicon (Apple M1 Max) with no NVIDIA GPU. The GPU serve benchmark requires an NVIDIA H100 or A100 with CUDA 12.x, so the measurement phase runs on an external CUDA machine.

## Bottleneck candidates

The integrated backend differs from the paged backend in exactly two places: the write-path slot mapping uses range arithmetic instead of a per-position block-table load, and the physical block placement uses contiguous ranges instead of free-list block tables. The integration is therefore an A/B instrument for these two differences. Four bottleneck candidates are identified in the codebase:

| Candidate | Location | Nature |
| :--- | :--- | :--- |
| 1. Per-step materialization copy | `_gather_block_tables_kernel` in `vllm/vllm/v1/worker/gpu/block_table.py`, launched from `gather_block_tables` and `model_runner.prepare_attn` | The whole block table is copied from the staged tensor to `input_block_tables` every step. Tagma can remove the staged tensor |
| 2. Decode block-table loads | `kernel_unified_attention` in `vllm/vllm/v1/attention/ops/triton_unified_attention.py` | One block-table load per KV tile, proportional to context length |
| 3. Prefill and chunked decode loads | `chunked_prefill_paged_decode` in `vllm/vllm/v1/attention/ops/chunked_prefill_paged_decode.py` and `context_attention_fwd` in the prefill kernels | Loads per KV block through the block table |
| 4. External attention backends | flash-attn, flashinfer and other fixed-ABI backends | The ABI is fixed; the materialized table stays in place |

## Redundant staged copy in tagma mode

In tagma mode `append_block_ids` stages the block ids into `block_tables` and updates the per-request range tables in the same call. `apply_staged_writes` writes the staged tensor, and `gather_block_tables` copies it into `input_block_tables` every step. The staged tensor exists only between these two calls. The range tables already hold the request layout, so the staged copy is redundant work in tagma mode.

The elimination candidate replaces the staged write and the gather with a range-expansion kernel that fills `input_block_tables` directly from the range tables, saving one full-table GPU write and one full-table GPU read per step. The candidate is implemented only if the GPU profile shows the materialization copy is a measurable share of step time.

## Execution plan

Each phase ends with a decision gate. The GPU measurement phase is the priority blocker because every later decision depends on its numbers.

### Phase 1: GPU measurement

Run the verification gate on a CUDA machine, then the serve benchmark, then profile the materialization copy.

- Run the four GPU write-path tests on the CUDA machine
- Run `run-vllm-tagma-benchmark.sh` with paged against tagma on the same card, on ShareGPT at 16K and 128K context, two repeats per backend in ABBA order
- Collect the per-run JSON, the server logs, the GPU memory samples, and the summary report
- Profile `_gather_block_tables_kernel` on the baseline run: the fraction of step time spent on the materialization copy, and the L2 and TLB behavior of the gather against sequential range access

Deliverable: the benchmark report in the syntagma devlogs with raw numbers, variance, machine details, and the fork commit.

Gate: parity or regression decides whether the integration is recorded as a measured negative result or whether the materialization copy removal proceeds.

### Phase 2: Materialization copy removal

The scope is limited to `block_table.py` in the vLLM fork: `append_block_ids`, `apply_staged_writes`, and `gather_block_tables`. `model_runner.py` and the attention backend ABI stay unchanged.

- Implement `_expand_ranges_to_block_table_kernel` that fills `input_block_tables` rows from the range tables, mirroring the scan pattern of `_compute_slot_mappings_tagma_kernel`
- Skip the staged write in tagma mode
- Extend the block-table tests and re-run the software-level gate

Gate: proceed only if the Phase 1 profile shows the materialization copy is a measurable share. If the profile shows noise, record the negative result and stop.

### Phase 3: Decision and recording

If the GPU benchmark measures parity, record the measured negative result following the decision rule in the KV cache report: the structural claim stands as a measured mechanism, and the write-path integration is not proposed upstream as a performance feature. Update the KV cache report and the homepage with the measured numbers.

### Phase 4: Read-path range arithmetic

Conditional on the Phase 1 profile showing a measurable gather share in the attention kernels. The scope is the vLLM-owned Triton kernels: `kernel_unified_attention` in `triton_unified_attention.py` and `context_attention_fwd` in the prefill kernels. The external backends keep the materialized table.

- Extend the attention metadata with the range tables and the per-request range count, an internal ABI change
- Replace the per-tile block-table load with the range scan
- Verify with the unified attention equivalence harness and end-to-end output parity

### Phase 5: TagmaVec search cost decomposition

Independent track, after the vLLM GPU measurement. Decompose the TagmaVec search cost following the assessment devlog, starting with the identity translation share (hypothesis 1) as the first component, before the deeper coordinate operations.

## GPU resource path

The local machine has no NVIDIA GPU. The candidate acquisition paths are the NVIDIA Inception program and the NVIDIA academic grant programs. Eligibility is unverified: Inception targets startups by public documentation, and SSCCS is a non-profit foundation. The eligibility check and the application are preparation tasks for Phase 1.

## Tracking

The integration work is tracked under ssccsorg/syntagma issue 52. The fork work proceeds on the SSCCS vLLM fork. This document records the plan; the issue tracks execution.

## References

- vLLM Tagma KV Cache report, ssccs docs, works/llms/vllm/kvcache
- vLLM Tagma KV Cache Benchmark Runbook, 2026-08-20-vllm-tagma-benchmark-runbook.md in this directory
- TagmaVec assessment, 2026-08-22-tagmavec-turbovec-assessment.md
