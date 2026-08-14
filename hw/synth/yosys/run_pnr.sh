#!/usr/bin/env bash
#
# Place and route the Tagma decoder demo for the Upduino 3.1
# (iCE40UP5K-SG48) and produce the bitstream and timing report.
#
# Flow: Yosys (synth_demo.ys) -> nextpnr-ice40 -> icepack -> icetime.
#
# Usage:
#   ./run_pnr.sh

set -euo pipefail
cd "$(dirname "$0")"

mkdir -p reports out

echo "--- yosys: demo top synthesis ---"
yosys -q -s synth_demo.ys

echo "--- nextpnr-ice40: place and route ---"
nextpnr-ice40 --up5k --package sg48 \
    --json out/tagma_demo_top.json \
    --pcf upduino31_demo.pcf \
    --asc out/tagma_demo_top.asc \
    --freq 12 2>&1 | tee reports/pnr.log | tail -20

echo "--- icetime: timing report ---"
icetime -d up5k -t out/tagma_demo_top.asc 2>&1 | tee reports/icetime.txt

echo "--- icepack: bitstream ---"
icepack out/tagma_demo_top.asc out/tagma_demo_top.bin

echo "bitstream: out/tagma_demo_top.bin"
echo "timing:    reports/icetime.txt"
