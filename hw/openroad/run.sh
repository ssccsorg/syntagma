#!/usr/bin/env bash
#
# Phase 4: OpenROAD Sky130 standard cell flow for the Tagma demo.
#
# Runs the ORFS (OpenROAD Flow Scripts) flow in the official Docker image
# for the registered demo top, and reports the pure decoder gate count
# against the Sky130 liberty with yosys stat -liberty.
#
# The ORFS image is x86_64. On Apple Silicon it runs under Rosetta, which
# may crash in some OpenROAD steps (illegal instruction). The intended
# execution environment is x86_64 CI (the p4 job in
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

echo "reports in results/"
