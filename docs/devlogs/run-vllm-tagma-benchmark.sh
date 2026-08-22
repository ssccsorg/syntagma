#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
#
# vLLM Tagma KV cache benchmark runner (Phase 5, ssccsorg/syntagma issue #52).
#
# Executes the methodology in 2026-08-20-vllm-tagma-benchmark-runbook.md on a
# CUDA machine: build, verification gate, then paged-versus-tagma measurement
# on ShareGPT at 16K and 128K context with alternating run order to cancel
# thermal drift. Requires an NVIDIA H100 or A100 and CUDA 12.x.
#
# Usage:
#   run-vllm-tagma-benchmark.sh --vllm-repo <path> --dataset <sharegpt.json> [options]
#
# The report is written to <result-dir>/report.md next to the per-run JSON
# files and the raw server logs.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

MODEL="meta-llama/Llama-3.1-8B-Instruct"
CONTEXT_LENS="16384,131072"
NUM_PROMPTS=200
OUTPUT_LEN=256
REQUEST_RATE="inf"
SEED=20260820
GPU_UTIL=0.85
PORT=8000
RESULT_DIR=""
VLLM_REPO=""
DATASET=""
SKIP_BUILD=0
SKIP_TESTS=0

usage() {
  cat <<'EOF'
Usage: run-vllm-tagma-benchmark.sh --vllm-repo <path> --dataset <sharegpt.json> [options]

Required:
  --vllm-repo <path>   ssccsorg/vllm checkout on branch 52-tagma-kv-vllm
  --dataset <path>     ShareGPT dataset JSON (e.g. ShareGPT_V3_unfiltered_cleaned_split.json)

Options:
  --model <id|path>    Model to serve, MHA only (default: meta-llama/Llama-3.1-8B-Instruct)
  --context-lens <csv> Context windows, comma separated (default: 16384,131072)
  --num-prompts <n>    Benchmark requests per run (default: 200)
  --output-len <n>     Generation length per request (default: 256)
  --request-rate <r>   Requests per second, inf for no throttling (default: inf)
  --seed <n>           Seed shared by both backends (default: 20260820)
  --gpu-memory-utilization <f>  KV cache sizing ratio (default: 0.85)
  --port <n>           Server port (default: 8000)
  --result-dir <path>  Where results are written (default: this devlog directory)
  --skip-build         Use the existing .venv inside the vllm repo
  --skip-tests         Skip the pytest verification gate
  -h, --help           Show this help
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --vllm-repo) VLLM_REPO="$2"; shift 2 ;;
    --dataset) DATASET="$2"; shift 2 ;;
    --model) MODEL="$2"; shift 2 ;;
    --context-lens) CONTEXT_LENS="$2"; shift 2 ;;
    --num-prompts) NUM_PROMPTS="$2"; shift 2 ;;
    --output-len) OUTPUT_LEN="$2"; shift 2 ;;
    --request-rate) REQUEST_RATE="$2"; shift 2 ;;
    --seed) SEED="$2"; shift 2 ;;
    --gpu-memory-utilization) GPU_UTIL="$2"; shift 2 ;;
    --port) PORT="$2"; shift 2 ;;
    --result-dir) RESULT_DIR="$2"; shift 2 ;;
    --skip-build) SKIP_BUILD=1; shift ;;
    --skip-tests) SKIP_TESTS=1; shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "error: unknown argument: $1" >&2; usage; exit 1 ;;
  esac
done

# ---- preflight -------------------------------------------------------------

if [[ -z "$VLLM_REPO" || -z "$DATASET" ]]; then
  echo "error: --vllm-repo and --dataset are required" >&2
  usage
  exit 1
fi
if ! command -v nvidia-smi >/dev/null 2>&1; then
  echo "error: nvidia-smi not found; this benchmark requires an NVIDIA GPU (H100 or A100)" >&2
  exit 1
fi
if [[ -z "$(nvidia-smi -L 2>/dev/null)" ]]; then
  echo "error: no NVIDIA GPU detected" >&2
  exit 1
fi
if [[ ! -d "$VLLM_REPO" ]]; then
  echo "error: vllm repo not found: $VLLM_REPO" >&2
  exit 1
fi
if [[ ! -f "$DATASET" ]]; then
  echo "error: ShareGPT dataset not found: $DATASET" >&2
  exit 1
fi
if [[ -z "$RESULT_DIR" ]]; then
  RESULT_DIR="$SCRIPT_DIR/bench-results"
fi
mkdir -p "$RESULT_DIR"

VENV="$VLLM_REPO/.venv"
if [[ "$SKIP_BUILD" -eq 0 ]]; then
  echo "[bench] building the vllm fork in $VLLM_REPO"
  (cd "$VLLM_REPO" && uv venv --python 3.12 .venv && uv pip install -e . --torch-backend=auto)
fi
if [[ ! -x "$VENV/bin/python" ]]; then
  echo "error: $VENV/bin/python missing; run without --skip-build or build the fork first" >&2
  exit 1
fi
PY="$VENV/bin/python"

echo "[bench] verifying the tagma extension loads"
"$PY" -c "from vllm.tagma import KVAllocator; assert KVAllocator(8).allocate(3) == (0, 3)"

if [[ "$SKIP_TESTS" -eq 0 ]]; then
  echo "[bench] running the verification gate"
  (cd "$VLLM_REPO" && "$PY" -m pytest -q tests/config/test_cache_config.py tests/v1/core/test_tagma_kv_cache_manager.py)
fi

GPU_NAME="$(nvidia-smi --query-gpu=name --format=csv,noheader | head -1)"
GPU_MEM_TOTAL="$(nvidia-smi --query-gpu=memory.total --format=csv,noheader,nounits | head -1)"
VLLM_COMMIT="$(git -C "$VLLM_REPO" rev-parse HEAD 2>/dev/null || echo unknown)"
SYNTAGMA_COMMIT="$(git -C "$SCRIPT_DIR/../.." rev-parse HEAD 2>/dev/null || echo unknown)"
SERVER_BIN="$VENV/bin/vllm"
if [[ ! -x "$SERVER_BIN" ]]; then
  echo "error: $SERVER_BIN missing; the editable install did not produce the vllm CLI" >&2
  exit 1
fi

# ---- helpers ---------------------------------------------------------------

SERVER_PID=""
cleanup() {
  if [[ -n "$SERVER_PID" ]] && kill -0 "$SERVER_PID" 2>/dev/null; then
    kill "$SERVER_PID" 2>/dev/null || true
    wait "$SERVER_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT

wait_for_server() {
  local tries=0
  until curl -sf "http://127.0.0.1:$PORT/health" >/dev/null 2>&1; do
    tries=$((tries + 1))
    if [[ $tries -gt 600 ]]; then
      echo "error: server did not become ready on port $PORT within 10 minutes" >&2
      return 1
    fi
    sleep 1
  done
}

sample_gpu_memory() {
  # Samples used memory every two seconds while the server runs and prints
  # the peak to stdout.
  local peak=0
  while kill -0 "$SERVER_PID" 2>/dev/null; do
    local used
    used="$(nvidia-smi --query-gpu=memory.used --format=csv,noheader,nounits | head -1)"
    if [[ "$used" =~ ^[0-9]+$ ]] && [[ "$used" -gt "$peak" ]]; then
      peak="$used"
    fi
    sleep 2
  done
  echo "$peak"
}

run_one() {
  # run_one <backend> <context_len> <run_id>
  local backend="$1"
  local ctx="$2"
  local run_id="$3"
  local server_log="$RESULT_DIR/$run_id.server.log"
  local gpu_sample_log="$RESULT_DIR/$run_id.gpu.txt"

  echo "[bench] run $run_id: backend=$backend context=$ctx"
  "$SERVER_BIN" serve "$MODEL" \
    --kv-cache-backend "$backend" \
    --max-model-len "$ctx" \
    --gpu-memory-utilization "$GPU_UTIL" \
    --port "$PORT" \
    >"$server_log" 2>&1 &
  SERVER_PID=$!
  if ! wait_for_server; then
    kill "$SERVER_PID" 2>/dev/null || true
    SERVER_PID=""
    return 1
  fi

  sample_gpu_memory >"$gpu_sample_log" &
  local sampler_pid=$!

  "$SERVER_BIN" bench serve \
    --backend openai \
    --base-url "http://127.0.0.1:$PORT/v1" \
    --model "$MODEL" \
    --dataset-name sharegpt \
    --dataset-path "$DATASET" \
    --request-rate "$REQUEST_RATE" \
    --num-prompts "$NUM_PROMPTS" \
    --sharegpt-output-len "$OUTPUT_LEN" \
    --seed "$SEED" \
    --percentile-metrics ttft,tpot,itl \
    --metric-percentiles 50,99 \
    --save-result \
    --result-filename "$run_id.json" \
    --result-dir "$RESULT_DIR" \
    >"$RESULT_DIR/$run_id.bench.log" 2>&1
  local bench_rc=$?

  kill "$SERVER_PID" 2>/dev/null || true
  wait "$SERVER_PID" 2>/dev/null || true
  SERVER_PID=""
  wait "$sampler_pid" 2>/dev/null || true

  if [[ $bench_rc -ne 0 ]]; then
    echo "error: benchmark client failed for $run_id (see $RESULT_DIR/$run_id.bench.log)" >&2
    return 1
  fi
  return 0
}

# ---- measurement -----------------------------------------------------------

IFS=',' read -r -a LENS <<< "$CONTEXT_LENS"
RUNS=()
for ctx in "${LENS[@]}"; do
  # Two repeats per backend per context window in ABBA order to cancel
  # thermal drift; the repeat index keeps the runs distinct on disk.
  RUNS+=("paged|$ctx|0")
  RUNS+=("tagma|$ctx|0")
  RUNS+=("tagma|$ctx|1")
  RUNS+=("paged|$ctx|1")
done

for entry in "${RUNS[@]}"; do
  IFS='|' read -r backend ctx rep <<< "$entry"
  run_id="ctx${ctx}-${backend}-r${rep}-seed${SEED}"
  if ! run_one "$backend" "$ctx" "$run_id"; then
    echo "error: run $run_id failed; aborting" >&2
    exit 1
  fi
done

# ---- report ----------------------------------------------------------------

REPORT="$RESULT_DIR/report.md"
"$PY" - "$RESULT_DIR" "$REPORT" "$GPU_NAME" "$GPU_MEM_TOTAL" "$VLLM_COMMIT" "$SYNTAGMA_COMMIT" "$MODEL" <<'PYEOF'
import json
import re
import sys

result_dir, report_path = sys.argv[1], sys.argv[2]
gpu_name, gpu_mem_total, vllm_commit, syntagma_commit, model = sys.argv[3:8]

rows = []
for path in sorted(__import__("pathlib").Path(result_dir).glob("ctx*-*.json")):
    run_id = path.stem
    m = re.match(r"ctx(\d+)-(\w+)-r(\d+)-seed(\d+)", run_id)
    if not m:
        continue
    ctx, backend, rep, seed = m.groups()
    with open(path) as f:
        data = json.load(f)
    ttft = {int(p): v for p, v in data.get("percentiles_ttft_ms", [])}
    tpot = {int(p): v for p, v in data.get("percentiles_tpot_ms", [])}
    kv_tokens = ""
    server_log = path.with_suffix(".server.log")
    if server_log.exists():
        text = server_log.read_text(errors="replace")
        hit = re.search(r"GPU KV cache size:\s*([\d,]+)\s*tokens", text)
        if hit:
            kv_tokens = hit.group(1).replace(",", "")
    peak_gpu = ""
    gpu_log = path.with_suffix(".gpu.txt")
    if gpu_log.exists():
        peak_gpu = gpu_log.read_text(errors="replace").strip()
    rows.append(
        (
            int(ctx), backend, int(rep),
            f"{ttft.get(50, float('nan')):.1f}", f"{ttft.get(99, float('nan')):.1f}",
            f"{tpot.get(50, float('nan')):.2f}", f"{tpot.get(99, float('nan')):.2f}",
            f"{data.get('output_throughput', float('nan')):.1f}",
            f"{data.get('total_token_throughput', float('nan')):.1f}",
            kv_tokens, peak_gpu, seed,
        )
    )

rows.sort(key=lambda r: (r[0], r[1], r[2]))

with open(report_path, "w") as f:
    f.write("# vLLM Tagma KV cache benchmark results\n\n")
    f.write(f"- Model: `{model}`\n")
    f.write(f"- GPU: {gpu_name} ({gpu_mem_total} MiB total)\n")
    f.write(f"- vllm fork commit: `{vllm_commit}`\n")
    f.write(f"- syntagma commit: `{syntagma_commit}`\n")
    f.write(f"- Seed: per-run seed column; two repeats per backend per context in ABBA order\n")
    f.write("\n| Context | Backend | Rep | TTFT p50 (ms) | TTFT p99 (ms) | TPOT p50 (ms) | TPOT p99 (ms) | Output tok/s | Total tok/s | KV tokens | Peak GPU mem (MiB) | Seed |\n")
    f.write("| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |\n")
    for r in rows:
        f.write("| " + " | ".join(str(x) for x in r) + " |\n")
    if not rows:
        f.write("\nNo result JSON files found in the result directory.\n")
PYEOF

echo "[bench] complete. Report: $REPORT"
