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
# Phase 4: OpenROAD Sky130 standard cell flow for the Tagma demo.
#
# Runs the ORFS (OpenROAD Flow Scripts) flow in the official Docker image
# for the registered demo top, and reports the pure decoder gate count
# against the Sky130 liberty with yosys stat -liberty.
#
# The ORFS image is x86_64. The flow runs on x86 natively and on Apple
# Silicon under Rosetta; the kepler-formal LEC step is disabled in the
# design config because the bundled binary crashes on both (see the
# derisking devlog). The intended
# execution environment is x86_64 CI (the hw job in
# .github/workflows/test.yml); run locally on x86 hardware or CI.
#
# Usage:
#   bash hw/openroad/run.sh

set -euo pipefail
cd "$(dirname "$0")"

IMG=openroad/orfs:latest
VOL=tagma_orfs_flow
FLOW=/OpenROAD-flow-scripts/flow
DESIGN_CONFIG=designs/sky130hd/tagma_demo/config.mk

mkdir -p results

echo "--- ORFS: Sky130hd standard cell flow for tagma_demo_top ---"
# The design RTL is mounted from hw/rtl into the ORFS src tree so the run
# always uses the current RTL (no committed copies to drift).
docker run --rm --platform linux/amd64 \
    -v "${VOL}:/OpenROAD-flow-scripts/flow" \
    -v "$(pwd)/designs:/OpenROAD-flow-scripts/flow/designs" \
    -v "$(pwd)/../rtl:/OpenROAD-flow-scripts/flow/designs/src/tagma_demo" \
    "${IMG}" bash -c "cd ${FLOW} && make DESIGN_CONFIG=${DESIGN_CONFIG}" \
    2>&1 | tee results/orfs.log | tail -25

echo "--- extract ORFS reports ---"
docker run --rm --platform linux/amd64 \
    -v "${VOL}:/OpenROAD-flow-scripts/flow" \
    -v "$(pwd)/results:/out" \
    "${IMG}" bash -c "cp -r /OpenROAD-flow-scripts/flow/results/sky130hd/tagma_demo /out/ 2>/dev/null || true"

echo "--- yosys: PDK gate count for the pure decoder ---"
docker run --rm --platform linux/amd64 \
    -v "${VOL}:/OpenROAD-flow-scripts/flow" \
    -v "$(pwd)/../rtl:/OpenROAD-flow-scripts/flow/designs/src/tagma_demo" \
    "${IMG}" bash -c '
        LIB=$(find /OpenROAD-flow-scripts/flow -name "sky130_fd_sc_hd__tt_025C_1v80.lib" | head -1)
        test -n "$LIB" || { echo "liberty not found after flow"; exit 1; }
        # synth before abc: the standalone yosys abc pass extracts nothing
        # from a raw \$mul netlist (proc; opt leaves the multiplier cells
        # unmappable), while the passes inside synth prepare them.
        yosys -p "read_verilog /OpenROAD-flow-scripts/flow/designs/src/tagma_demo/tagma_decoder.v;
                  synth -top tagma_decoder;
                  read_liberty -lib $LIB; abc -liberty $LIB; opt;
                  stat -liberty $LIB"
    ' 2>&1 | tee results/sky130_decoder_stat.txt | tail -25

echo "--- power at the demo workload's input activity ---"
# The design config names power_activity.tcl as POST_FINAL_REPORT_TCL, so the
# flow sources it at the end of its report step, in the session where the
# design, the liberty, the sdc, the derate, the setRC, and the SPEF are already
# in place. The hook's default-activity report therefore repeats the flow's own
# number and its annotated report differs only by the input activity. The hook
# writes $RESULTS_DIR/power_activity.rpt into the flow volume, and this step
# reads it out, so a missing or failed hook fails the gate. See
# designs/sky130hd/tagma_demo/power_activity.tcl and
# docs/devlogs/hw/2026-09-28-power-activity.md.
docker run --rm --platform linux/amd64 \
    -v "${VOL}:/OpenROAD-flow-scripts/flow" \
    "${IMG}" bash -c '
        set -eo pipefail
        RPT=$(find /OpenROAD-flow-scripts/flow -name "power_activity.rpt" -print -quit)
        test -n "$RPT" || { echo "no power_activity.rpt in the flow volume, so the report step did not source the hook"; exit 1; }
        echo "report: $RPT"
        cat "$RPT"
    ' 2>&1 | tee results/power_activity.txt | tail -40

echo "reports in results/"
