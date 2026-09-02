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
    check_cpp
    check_hw
}

build_and_test() {
    (cd sw/rust && cargo build --release)
    (cd sw/rust && cargo test --release)
    check_cpp
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
