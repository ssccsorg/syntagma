#!/bin/bash
set -e
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
COMMIT_HASH=$(cd "$(dirname "$0")/.." && git rev-parse --short HEAD 2>/dev/null || echo "unknown")

cd "$(dirname "$0")"
SCRIPT_DIR="$(pwd)"
RESULT_DIR="$SCRIPT_DIR/result"
mkdir -p "$RESULT_DIR"

echo "=== Tagma Benchmark Suite ==="
echo "Timestamp: $TIMESTAMP"
echo "Commit:    $COMMIT_HASH"
echo ""

echo "=== Running spatial benchmarks ==="
cargo bench --bench bench -- spatial 2>&1 | tee "$RESULT_DIR/output-${TIMESTAMP}.txt" || true

echo "=== Running kv benchmarks (may take several minutes) ==="
cargo bench --bench bench -- kv 2>&1 | tee -a "$RESULT_DIR/output-${TIMESTAMP}.txt" || true

echo "=== Running set benchmarks ==="
cargo bench --bench bench -- set 2>&1 | tee -a "$RESULT_DIR/output-${TIMESTAMP}.txt" || true

echo "=== Exporting results ==="
python3 "$SCRIPT_DIR/export_results.py" "$RESULT_DIR/bench-${TIMESTAMP}-${COMMIT_HASH}.json"

echo ""
echo "=== Done ==="
echo "Raw output:  $RESULT_DIR/output-${TIMESTAMP}.txt"
echo "JSON result: $RESULT_DIR/bench-${TIMESTAMP}-${COMMIT_HASH}.json"
