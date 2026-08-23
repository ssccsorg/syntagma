#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
#
# llama.cpp Tagma KV cache benchmark runner (Phase A baseline, syntagma issue 53).
#
# Single bench code point: llama-bench. Single collection point: this script.
# Measures the baseline (current master) on a CPU-only build (GGML_METAL=OFF),
# the GPU-free proof surface. The KV cache dimension is the prompt length,
# which llama-bench derives the context from; the tagma variant later replaces
# the binary path and re-runs the same runner, keeping the code point and the
# collection point identical for the A/B comparison.
#
# Usage:
#   run-llamacpp-tagma-benchmark.sh --llama-repo <path> --model <gguf> [options]
#
# The report is written to <result-dir>/report.md next to the per-run JSON
# files and the raw benchmark logs.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

PROMPT_LENS="32768,131072"
N_OUTPUT=256
REPEATS=2
N_GPU_LAYERS=0
RESULT_DIR=""
LLAMA_REPO=""
MODEL=""

usage() {
  cat <<'EOF'
Usage: run-llamacpp-tagma-benchmark.sh --llama-repo <path> --model <gguf> [options]

Required:
  --llama-repo <path>   ssccsorg/llama.cpp checkout with a build/bin/llama-bench
  --model <path>        GGUF model file (small model recommended for CPU)

Options:
  --prompt-lens <csv>   Prompt (KV) lengths, comma separated (default: 32768,131072)
  --n-output <tokens>   Generated tokens per run (default: 256)
  --repeats <n>         llama-bench repetitions per context (default: 2)
  --n-gpu-layers <n>    GPU layers, 0 for the CPU-only baseline (default: 0)
  --result-dir <path>   Where results are written (default: this devlog directory)
  -h, --help            Show this help
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --llama-repo) LLAMA_REPO="$2"; shift 2 ;;
    --model) MODEL="$2"; shift 2 ;;
    --prompt-lens) PROMPT_LENS="$2"; shift 2 ;;
    --n-output) N_OUTPUT="$2"; shift 2 ;;
    --repeats) REPEATS="$2"; shift 2 ;;
    --n-gpu-layers) N_GPU_LAYERS="$2"; shift 2 ;;
    --result-dir) RESULT_DIR="$2"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "error: unknown argument: $1" >&2; usage; exit 1 ;;
  esac
done

# ---- preflight -------------------------------------------------------------

if [[ -z "$LLAMA_REPO" || -z "$MODEL" ]]; then
  echo "error: --llama-repo and --model are required" >&2
  usage
  exit 1
fi
BENCH="$LLAMA_REPO/build/bin/llama-bench"
if [[ ! -x "$BENCH" ]]; then
  echo "error: $BENCH missing; build the llama.cpp fork first" >&2
  exit 1
fi
if [[ ! -f "$MODEL" ]]; then
  echo "error: model not found: $MODEL" >&2
  exit 1
fi
if [[ -z "$RESULT_DIR" ]]; then
  RESULT_DIR="$SCRIPT_DIR/bench-results-llamacpp"
fi
mkdir -p "$RESULT_DIR"

COMMIT="$(git -C "$LLAMA_REPO" rev-parse HEAD 2>/dev/null || echo unknown)"

# ---- helpers ---------------------------------------------------------------

run_one() {
  # run_one <prompt_len>
  local prompt_len="$1"
  local run_id="ctx${prompt_len}"
  echo "[bench] run $run_id: prompt=$prompt_len output=$N_OUTPUT repeats=$REPEATS"
  "$BENCH" -m "$MODEL" -p "$prompt_len" -n "$N_OUTPUT" -r "$REPEATS" \
    -ngl "$N_GPU_LAYERS" -o json \
    >"$RESULT_DIR/$run_id.json" 2>"$RESULT_DIR/$run_id.log"
}

# ---- measurement -----------------------------------------------------------

IFS=',' read -r -a LENS <<< "$PROMPT_LENS"
for prompt_len in "${LENS[@]}"; do
  run_one "$prompt_len"
done

# ---- report ----------------------------------------------------------------

REPORT="$RESULT_DIR/report.md"
python3 - "$RESULT_DIR" "$REPORT" "$MODEL" "$COMMIT" "$N_GPU_LAYERS" "$REPEATS" <<'PYEOF'
import json
import pathlib
import sys

result_dir, report_path = sys.argv[1], sys.argv[2]
model, commit, n_gpu_layers, repeats = sys.argv[3:7]

rows = []
for path in sorted(pathlib.Path(result_dir).glob("ctx*.json")):
    run_id = path.stem
    ctx = run_id[3:]
    with open(path) as f:
        data = json.load(f)
    pp = tg = None
    for r in data:
        if r.get("n_prompt", 0) > 0 and r.get("n_gen", 0) == 0:
            pp = r.get("avg_ts")
        elif r.get("n_gen", 0) > 0:
            tg = r.get("avg_ts")
    rows.append(
        (
            int(ctx),
            f"{pp:.1f}" if pp is not None else "n/a",
            f"{tg:.2f}" if tg is not None else "n/a",
        )
    )

rows.sort(key=lambda r: r[0])

with open(report_path, "w") as f:
    f.write("# llama.cpp Tagma KV cache benchmark results\n\n")
    f.write(f"- Model: `{model}`\n")
    f.write(f"- llama.cpp fork commit: `{commit}`\n")
    f.write(f"- GPU layers: {n_gpu_layers} (CPU-only build, GGML_METAL=OFF)\n")
    f.write(f"- llama-bench repetitions: {repeats}\n\n")
    f.write("| Prompt (KV) length | Prompt tok/s (pp) | Decode tok/s (tg) |\n")
    f.write("| --- | --- | --- |\n")
    for ctx, pp, tg in rows:
        f.write(f"| {ctx} | {pp} | {tg} |\n")
    if not rows:
        f.write("\nNo result JSON files found in the result directory.\n")
PYEOF

echo "[bench] complete. Report: $REPORT"
