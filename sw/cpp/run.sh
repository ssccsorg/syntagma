#!/usr/bin/env bash
set -euo pipefail
#
# Tagma C++ implementation — single entry point.
#
# Usage:
#   ./run.sh                 # configure -> build -> test
#   ./run.sh --bench         # build + run the benchmark suite
#   ./run.sh --help
#
# The default GitHub CI image (ubuntu-latest) ships cmake and g++, so no
# extra toolchain setup is required.
#
cd "$(dirname "$0")"

case "${1:-}" in
    --bench|-b)
        cmake -S . -B build -DCMAKE_BUILD_TYPE=Release
        cmake --build build --target tagma_bench
        RESULT_DIR="bench/result"
        mkdir -p "$RESULT_DIR"
        TIMESTAMP=$(date +%Y%m%d-%H%M%S)
        COMMIT_HASH=$(git rev-parse --short HEAD 2>/dev/null || echo unknown)
        ./build/bench/tagma_bench \
            --json "$RESULT_DIR/bench-${TIMESTAMP}-${COMMIT_HASH}.json" \
            --commit "$COMMIT_HASH" \
            --timestamp "$TIMESTAMP"
        exit 0
        ;;
    --help|-h)
        echo "Usage: ./run.sh [--help]"
        echo "       ./run.sh            # configure -> build -> test"
        echo "       ./run.sh --bench    # build + run the benchmark suite"
        exit 0
        ;;
esac

cmake -S . -B build -DCMAKE_BUILD_TYPE=Release
cmake --build build
ctest --test-dir build --output-on-failure
