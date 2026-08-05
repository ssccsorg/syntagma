#!/usr/bin/env bash
set -euo pipefail
#
# Tagma C++ implementation — single entry point.
#
# Usage:
#   ./run.sh                 # configure → build → test
#   ./run.sh --help
#
# The default GitHub CI image (ubuntu-latest) ships cmake and g++, so no
# extra toolchain setup is required.
#
cd "$(dirname "$0")"

case "${1:-}" in
    --help|-h)
        echo "Usage: ./run.sh [--help]"
        exit 0
        ;;
esac

cmake -S . -B build -DCMAKE_BUILD_TYPE=Release
cmake --build build
ctest --test-dir build --output-on-failure
