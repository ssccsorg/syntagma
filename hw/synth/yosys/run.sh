#!/usr/bin/env bash
# SPDX-License-Identifier: CERN-OHL-P-2.0
# Copyright (C) 2026 Taeho Lee
#
# This source describes Open Hardware and is licensed under the CERN-OHL-P v2.
# You may redistribute and modify this documentation and make products using
# it under the terms of the CERN-OHL-P v2 (https://cern.ch/cern-ohl). This
# documentation is distributed WITHOUT ANY EXPRESS OR IMPLIED WARRANTY,
# INCLUDING OF MERCHANTABILITY, SATISFACTORY QUALITY AND FITNESS FOR A
# PARTICULAR PURPOSE. Please see the CERN-OHL-P v2 for applicable conditions.
#
#
# Run all Yosys synthesis flows for the Tagma decoder and write reports.
#
# Usage:
#   ./run.sh            # generic, gate-level, and iCE40 flows
#   ./run.sh generic    # only the generic flow
#   ./run.sh gates      # only the decoder gate-level flow
#   ./run.sh compose    # only the compose gate-level flow
#   ./run.sh dist       # only the distance gate-level flow
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
    compose) run synth_compose_gates ;;
    dist)    run synth_dist_gates ;;
    ice40)   run synth_ice40 ;;
    all)
        run synth_generic
        run synth_gates
        run synth_ice40
        run synth_compose_gates
        run synth_dist_gates
        ;;
    *)
        echo "unknown flow: $flow" >&2
        exit 1
        ;;
esac

echo "reports written to reports/"
