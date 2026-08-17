#!/usr/bin/env bash
#
# Run all Yosys synthesis flows for the Tagma decoder and write reports.
#
# Usage:
#   ./run.sh            # generic, gate-level, and iCE40 flows
#   ./run.sh generic    # only the generic flow
#   ./run.sh gates      # only the gate-level flow
#   ./run.sh ice40      # only the iCE40 flow

set -euo pipefail
cd "$(dirname "$0")"

mkdir -p reports out

flow="${1:-all}"

run() {
    echo "--- yosys: $1 ---"
    yosys -q -s "$1.ys"
}

case "$flow" in
    generic) run synth_generic ;;
    gates)   run synth_gates ;;
    ice40)   run synth_ice40 ;;
    all)
        run synth_generic
        run synth_gates
        run synth_ice40
        ;;
    *)
        echo "unknown flow: $flow" >&2
        exit 1
        ;;
esac

echo "reports written to reports/"
