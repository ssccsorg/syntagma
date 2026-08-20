# vLLM Tagma KV Cache Benchmark Runbook

## Purpose

This runbook measures the tagma KV cache backend on the SSCCS vLLM fork against the stock paged backend. The claim under test is that the tagma write path derives physical blocks by range arithmetic without a block-table load. The measurement decides whether the deferred read-path variant for vLLM-owned Triton kernels is worth implementing.

The backend is tracked in `ssccsorg/syntagma` issue #52 and implemented on `ssccsorg/vllm` branch `52-tagma-kv-vllm`.

## Environment

| Item | Requirement |
| :--- | :--- |
| GPU | NVIDIA H100 or A100, same card for both runs |
| Driver | CUDA 12.x compatible |
| Python | 3.12 via `uv` |
| Dataset | ShareGPT, 16K and 128K context windows |

## Build

```bash
uv venv --python 3.12 .venv
source .venv/bin/activate
uv pip install -e . --torch-backend=auto
```

The full build compiles the `vllm.tagma` host extension (pure C++, SABI 3.11). Verify the extension loads:

```bash
python -c "from vllm.tagma import KVAllocator; print(KVAllocator(8).allocate(3))"
```

## Test verification

```bash
pytest tests/config/test_cache_config.py
pytest tests/v1/core/test_tagma_kv_cache_manager.py
```

The config tests cover the `--kv-cache-backend` flag. The manager tests cover contiguous range allocation, grow-in-place extension, prefix cache hits, cache eviction, and the common-prefix statistic. They run on CPU with the built extension and do not require a GPU.

## Benchmark methodology

| Item | Definition |
| :--- | :--- |
| Model | A multi-head attention model (Llama class). The tagma backend rejects MLA, Mamba, and hybrid caches with `NotImplementedError`, so those are out of scope |
| Baseline | Stock paged backend, the default (`--kv-cache-backend paged`) |
| Treatment | Tagma backend (`--kv-cache-backend tagma`) |
| Workloads | ShareGPT, 16K and 128K maximum context |
| Metrics | p50 and p99 time to first token, decode latency per token, throughput in tokens per second, KV cache memory footprint |
| Isolation | Same machine, same model checkpoint, same seed; alternate runs to cancel thermal drift |

Example serve commands:

```bash
vllm serve <model> --kv-cache-backend paged --max-model-len 16384
vllm serve <model> --kv-cache-backend tagma --max-model-len 16384
```

## What the read-path decision needs

The attention backends consume the materialized `block_table` in this version. The deferred item is range arithmetic inside the vLLM-owned Triton kernels (`unified_attention`, `context_attention_fwd`). Before implementing it, profile the block-table gather in those kernels on the baseline run:

- Fraction of kernel time spent loading `block_table` entries
- L2 and TLB behavior of the gather versus sequential range access

If the gather is a measurable share, the read-path variant is justified. If it is noise, the write-path result stands alone and the read path stays on the materialized table.

## Success criteria

The criteria are measurements, not assumptions:

| Metric | Baseline (paged) | Tagma |
| :--- | :--- | :--- |
| Write-path slot mapping | Block-table gather per position | Range arithmetic, no gather |
| TTFT and decode latency | Measured | Measured, compared |
| Throughput at 16K and 128K | Measured | Measured, compared |
| KV cache footprint | Measured | Measured, compared |

The CERN ROOT-Bypass numbers (69.0x coordinate, 181.7x mapped) are the measured precedent of the same structural pattern on a different workload. They do not project a vLLM speedup; the vLLM numbers come from this runbook.

## Recording

Record the results in this directory following the measured-data conventions of the other devlogs: methodology, raw numbers, variance, machine details, and the exact commit of the fork. The report accompanies the pull request proposal.
