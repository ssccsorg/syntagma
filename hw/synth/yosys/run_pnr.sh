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
# Place and route the demo for one board, then produce that board's bitstream
# and timing report. The boards do not share a toolchain, so each case below is
# a complete flow and neither branch reads the other board's constraints.
#
#   board          synthesis      place and route    packing
#   upduino31      synth_ice40    nextpnr-ice40      icepack
#   icesugar_pro   synth_ecp5     nextpnr-ecp5       ecppack
#
# nextpnr-ecp5 is not packaged by Homebrew. Put OSS CAD Suite on PATH before
# running the icesugar_pro case.
#
# Environment note: icetime needs the iCE40 chipdb files discoverable.
# On macOS Homebrew, the icestorm chipdb lives under
#   $(brew --prefix icestorm)/share/icestorm/chipdb
# while icetime looks under share/icebox; a symlink may be required:
#   ln -s $(brew --prefix icestorm)/share/icestorm/chipdb \
#         $(brew --prefix)/share/icebox
#
# Usage:
#   ./run_pnr.sh [upduino31|icesugar_pro]

set -euo pipefail
cd "$(dirname "$0")"

BOARD="${1:-upduino31}"
BOARDS="../../boards"

mkdir -p reports out

case "$BOARD" in
upduino31)
    echo "--- yosys: demo top synthesis (iCE40) ---"
    yosys -q -s synth_demo.ys

    echo "--- nextpnr-ice40: place and route ---"
    nextpnr-ice40 --up5k --package sg48 \
        --json out/tagma_demo_top.json \
        --pcf "$BOARDS/upduino31/tagma_demo.pcf" \
        --asc out/tagma_demo_top.asc \
        --freq 12 2>&1 | tee reports/pnr.log | tail -20

    echo "--- icetime: timing report ---"
    icetime -d up5k -t out/tagma_demo_top.asc 2>&1 | tee reports/icetime.txt

    echo "--- icepack: bitstream ---"
    icepack out/tagma_demo_top.asc out/tagma_demo_top.bin

    echo "--- icebox_vlog: recover the placed-and-routed logic for simulation ---"
    icebox_vlog -c -p "$BOARDS/upduino31/tagma_demo.pcf" -n tagma_demo_top_postpnr \
        out/tagma_demo_top.asc > out/tagma_demo_top_postpnr.v

    echo "bitstream: out/tagma_demo_top.bin"
    echo "timing:    reports/icetime.txt"
    echo "recovered: out/tagma_demo_top_postpnr.v"
    ;;

icesugar_pro)
    # No ECP5 equivalent of icebox_vlog exists, so this flow ends at the
    # bitstream and the nextpnr timing summary.
    echo "--- yosys: bring-up synthesis (ECP5) ---"
    yosys -q -p "read_verilog -sv $BOARDS/icesugar_pro/bringup.v; \
        synth_ecp5 -top bringup -json out/bringup.json"

    echo "--- nextpnr-ecp5: place and route ---"
    nextpnr-ecp5 --25k --package CABGA256 --speed 6 \
        --json out/bringup.json \
        --lpf "$BOARDS/icesugar_pro/bringup.lpf" \
        --textcfg out/bringup.config \
        --freq 25 2>&1 | tee reports/pnr_icesugar_pro.log | tail -20

    echo "--- ecppack: bitstream ---"
    ecppack out/bringup.config out/bringup.bit

    echo "bitstream: out/bringup.bit"
    echo "config:    out/bringup.config"
    echo "timing:    reports/pnr_icesugar_pro.log"
    ;;

*)
    echo "unknown board: $BOARD (expected upduino31 or icesugar_pro)" >&2
    exit 1
    ;;
esac
