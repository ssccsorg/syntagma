#!/usr/bin/env bash
set -euo pipefail
#
# Tagma Java implementation - single entry point.
#
# Usage:
#   ./run.sh                 # build + test (mvn verify)
#   ./run.sh --help
#
# Requires JDK 21 and Maven. The CI java job installs both with
# actions/setup-java (temurin 21) plus the Maven cache.
#
cd "$(dirname "$0")"

case "${1:-}" in
    --help|-h)
        echo "Usage: ./run.sh [--help]"
        echo "       ./run.sh   # maven verify (build + test)"
        exit 0
        ;;
esac

echo "--- java: maven build + test ---"
exec mvn -B verify
