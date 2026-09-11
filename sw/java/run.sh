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
    --bench|-b)
        # The benchmark suite materializes CoordSpaceM windows of up to 2 GiB
        # of direct memory, and exec:java runs inside this JVM.
        export MAVEN_OPTS="${MAVEN_OPTS:-} -XX:MaxDirectMemorySize=3g"
        # exec:java resolves the module dependencies from the local
        # repository, so package the reactor first. Tests are skipped here
        # because the benchmark profile is the work in this path.
        mvn -B -q -DskipTests install
        RESULT_DIR="bench/result"
        mkdir -p "$RESULT_DIR"
        TIMESTAMP=$(date +%Y%m%d-%H%M%S)
        COMMIT_HASH=$(git rev-parse --short HEAD 2>/dev/null || echo unknown)
        mvn -B -q -f bench/pom.xml compile exec:java \
            -Dexec.args="--json $RESULT_DIR/bench-${TIMESTAMP}-${COMMIT_HASH}.json --commit $COMMIT_HASH --timestamp $TIMESTAMP"
        exit 0
        ;;
    --help|-h)
        echo "Usage: ./run.sh [--help]"
        echo "       ./run.sh            # maven verify (build + test)"
        echo "       ./run.sh --bench    # build + run the benchmark suite"
        exit 0
        ;;
esac

echo "--- java: maven build + test ---"
exec mvn -B verify
