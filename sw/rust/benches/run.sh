#!/bin/bash
set -e
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
COMMIT_HASH=$(cd "$(dirname "$0")/.." && git rev-parse --short HEAD 2>/dev/null || echo "unknown")
RESULT_DIR="$(dirname "$0")/result"
mkdir -p "$RESULT_DIR"

echo "=== Tagma Benchmark Suite ==="
echo "Timestamp: $TIMESTAMP"
echo "Commit:    $COMMIT_HASH"
echo ""

cd "$(dirname "$0")"

# Run benchmark groups
echo "=== Running spatial benchmarks ==="
cargo bench --bench bench -- spatial 2>&1 | tee "$RESULT_DIR/output-${TIMESTAMP}.txt"

echo "=== Running kv benchmarks (may take several minutes) ==="
cargo bench --bench bench -- kv 2>&1 | tee -a "$RESULT_DIR/output-${TIMESTAMP}.txt"

echo "=== Running set benchmarks ==="
cargo bench --bench bench -- set 2>&1 | tee -a "$RESULT_DIR/output-${TIMESTAMP}.txt"

# Export criterion results to summary JSON
echo "=== Exporting results ==="
python3 "$(dirname "$0")/export_results.py" "$RESULT_DIR/bench-${TIMESTAMP}-${COMMIT_HASH}.json"

echo ""
echo "=== Done ==="
echo "Raw output:  $RESULT_DIR/output-${TIMESTAMP}.txt"
echo "JSON result: $RESULT_DIR/bench-${TIMESTAMP}-${COMMIT_HASH}.json"
