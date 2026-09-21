#!/usr/bin/env bash
set -euo pipefail
#
# tagma — Single entry point
#
# Usage:
#   ./run.sh                 # Full pipeline: check → build → test
#   ./run.sh --check         # fmt → clippy → build → test (strict)
#   ./run.sh --fix           # auto-fix → build → test
#   ./run.sh --bench         # build + test + core benchmarks
#   ./run.sh --doc           # build documentation
#   ./run.sh --help
#

cd "$(dirname "$0")"
export RUSTFLAGS="-D warnings"

# ── Helpers ───────────────────────────────────────────────────────────

CPP_DIR="sw/cpp"
CPP_BUILD="$CPP_DIR/build"

check_cpp() {
    echo "--- c++ core: configure + build + test ---"
    cmake -S "$CPP_DIR" -B "$CPP_BUILD" -DCMAKE_BUILD_TYPE=Release
    cmake --build "$CPP_BUILD"
    ctest --test-dir "$CPP_BUILD" --output-on-failure
}

check_java() {
    echo "--- java core: maven build + test ---"
    if ! command -v mvn >/dev/null 2>&1 || ! command -v java >/dev/null 2>&1; then
        echo "  skipped (mvn and/or java not installed)"
        return 0
    fi
    # The Java port targets Java 21. Runner images may ship an older default
    # JDK, so only run when the active JDK major version is 21 or newer.
    java_major=$(java -version 2>&1 | sed -n 's/.*version "\([0-9]*\)[^"]*".*/\1/p')
    if [ -z "$java_major" ] || [ "$java_major" -lt 21 ]; then
        echo "  skipped (JDK 21+ required, found $(java -version 2>&1 | head -1))"
        return 0
    fi
    (cd sw/java && mvn -B verify)
}

check_java_bench() {
    echo "--- java bench: maven build + benchmark suite ---"
    if ! command -v mvn >/dev/null 2>&1 || ! command -v java >/dev/null 2>&1; then
        echo "  skipped (mvn and/or java not installed)"
        return 0
    fi
    # The Java port targets Java 21. Runner images may ship an older default
    # JDK, so only run when the active JDK major version is 21 or newer.
    java_major=$(java -version 2>&1 | sed -n 's/.*version "\([0-9]*\)[^"]*".*/\1/p')
    if [ -z "$java_major" ] || [ "$java_major" -lt 21 ]; then
        echo "  skipped (JDK 21+ required, found $(java -version 2>&1 | head -1))"
        return 0
    fi
    (cd sw/java && ./run.sh --bench)
}

check_hw() {
    echo "--- hw: RTL simulation + synthesis ---"
    if ! command -v verilator >/dev/null 2>&1 || ! command -v yosys >/dev/null 2>&1 || ! command -v python3 >/dev/null 2>&1; then
        echo "  skipped (verilator, yosys, and/or python3 not installed)"
        return 0
    fi
    make -C hw check
}

check_checks() {
    (cd sw/rust && cargo fmt --check)
    # default feature set (alloc): tree, dense, set types
    echo "--- clippy (default features) ---"
    (cd sw/rust && cargo clippy --all-targets)
    # mmap feature: CoordSpaceM (N>=3 mmap-backed dense)
    echo "--- clippy (mmap feature) ---"
    (cd sw/rust && cargo clippy --all-targets --features mmap)
    echo "--- build + test (default features) ---"
    (cd sw/rust && cargo build --release)
    (cd sw/rust && cargo test --release)
    echo "--- build + test (mmap feature) ---"
    (cd sw/rust && cargo build --release --features mmap)
    (cd sw/rust && cargo test --release --features mmap)
    # no_alloc: verify Coord, CoordPath, CoordSet, CoordSpace compile
    # without heap allocator (core types only).
    echo "--- no_alloc build + test ---"
    (cd sw/rust && cargo build --release --no-default-features)
    (cd sw/rust && cargo test --release --no-default-features)
    # MCU: the tagma family (core/geo/map) must compile for a std-less
    # target with the default alloc feature, which is what the no_std
    # storage path (chton, nex) consumes on device.
    echo "--- riscv32imac-unknown-none-elf check (MCU target) ---"
    (cd sw/rust && cargo check -p tagma-core -p tagma-geo -p tagma-map --target riscv32imac-unknown-none-elf)
    # The no-allocator member is linked rather than only checked. The link fails
    # with "no global memory allocator found but one is required" if anything
    # underneath reaches for the allocator, so the property is a fact about the
    # artifact rather than a claim. linkcheck is its own workspace (see the exclude
    # note in sw/rust/Cargo.toml), because a build that also selects tagma-geo or
    # tagma-map would unify tagma-core's alloc on.
    echo "--- no-allocator link (no global allocator, no OS) ---"
    (cd sw/rust/verify/linkcheck && cargo fmt --check)
    (cd sw/rust/verify/linkcheck && cargo clippy --target riscv32imac-unknown-none-elf -- -D warnings)
    (cd sw/rust/verify/linkcheck && cargo build --target riscv32imac-unknown-none-elf)
    check_cpp
    check_java
    check_hw
}

build_and_test() {
    (cd sw/rust && cargo build --release)
    (cd sw/rust && cargo test --release)
    check_cpp
    check_java
}

auto_fix() {
    (cd sw/rust && cargo fmt --all)
    (cd sw/rust && cargo clippy --fix --allow-dirty 2>&1 || true)
    (cd sw/rust && cargo fix --allow-dirty 2>&1 || true)
    (cd sw/rust && cargo fmt --all)
}

build_docs() {
    (cd sw/rust && cargo doc --no-deps)
}

# ── Dispatch ──────────────────────────────────────────────────────────

case "${1:-}" in
    --check|check)
        check_checks
        ;;
    --fix|fix)
        auto_fix
        build_and_test
        ;;
    --bench|bench)
        build_and_test
        echo "--- running core benchmarks ---"
        (cd sw/rust && cargo bench --features mmap -- "inserts|lookup|n_scaling|n2_comparison|spatial|edge|hw" 2>&1 | tail -20)
        check_java_bench
        ;;
    --doc|doc)
        build_docs
        ;;
    --help|-h)
        echo "Usage: ./run.sh [--check|--fix|--bench|--doc|--help]"
        exit 0
        ;;
    *)
        auto_fix
        check_checks
        ;;
esac
