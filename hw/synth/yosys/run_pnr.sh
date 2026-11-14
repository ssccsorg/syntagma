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
#   board          design        synthesis      place and route    packing
#   upduino31      tagma_demo    synth_ice40    nextpnr-ice40      icepack
#   icesugar_pro   bringup       synth_ecp5     nextpnr-ecp5       ecppack
#   icesugar_pro   tagma_demo    synth_ecp5     nextpnr-ecp5       ecppack
#   icesugar_pro   stream        synth_ecp5     nextpnr-ecp5       ecppack
#   icesugar_pro   verify        synth_ecp5     nextpnr-ecp5       ecppack
#   icesugar_pro   stream_check  synth_ecp5     nextpnr-ecp5       ecppack
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
#   ./run_pnr.sh [upduino31|icesugar_pro] [bringup|tagma_demo|stream|stream_check|verify|volume]

set -euo pipefail
cd "$(dirname "$0")"

BOARD="${1:-upduino31}"
DESIGN="${2:-bringup}"
NAME=""
REPORT="${DESIGN}"
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
    case "$DESIGN" in
    bringup)
        SRC="$BOARDS/icesugar_pro/bringup.v"
        TOP=bringup
        LPF="$BOARDS/icesugar_pro/bringup.lpf"
        PARAMS=""
        ;;
    tagma_demo)
        SRC="../../rtl/tagma_decoder.v ../../rtl/demo/tagma_demo_top.v"
        TOP=tagma_demo_top
        LPF="$BOARDS/icesugar_pro/tagma_demo.lpf"
        # The RGB LED lights on a high level, unlike the Upduino's.
        PARAMS="chparam -set LED_ACTIVE_LOW 0 tagma_demo_top"
        ;;
    stream)
        SRC="../../rtl/tagma_decoder_pipe.v ../../rtl/demo/tagma_pipe_top.v"
        TOP=tagma_pipe_top
        LPF="$BOARDS/icesugar_pro/stream.lpf"
        # The code source is a counter, so the reported path is a register to
        # register path through the decoder rather than a cross-domain one.
        PARAMS=""
        ;;
    stream_check)
        SRC="../../rtl/tagma_decoder.v ../../rtl/tagma_decoder_pipe.v ../../rtl/demo/tagma_pipe_pll.v ../../rtl/demo/tagma_pipe_check_top.v"
        TOP=tagma_pipe_check_top
        LPF="$BOARDS/icesugar_pro/stream_check.lpf"
        # Runs from the PLL output so the check covers the pipeline above the
        # board clock, and reports the three observations on the RGB balls.
        PARAMS=""
        ;;
    verify)
        # The output rate is a build parameter, because the point of this image
        # is to find where the decode stops matching as the clock goes up.
        MHZ="${VERIFY_MHZ:-40}"
        PLL="out/pipe_pll_${MHZ}.v"
        echo "--- ecppll: 25 MHz to $MHZ MHz ---"
        ecppll -i 25 -o "$MHZ" -f "$PLL" --module pipe_pll \
            --clkin_name clk_in --clkout0_name clk >/dev/null
        SRC="../../rtl/tagma_decoder_pipe.v $PLL ../../rtl/demo/tagma_pipe_verify_top.v"
        TOP=tagma_pipe_verify_top
        LPF="$BOARDS/icesugar_pro/verify.lpf"
        # The expected checksum is derived from the anchors tagma_core writes,
        # so the die is checked against the reference rather than against itself.
        CHECKSUM=$(python3 ../../tools/tagma_checksum.py ../../rtl/golden_anchors.hex)
        PARAMS="chparam -set EXPECTED 32'h$CHECKSUM $TOP"
        NAME="${TOP}_${MHZ}mhz"
        REPORT="verify_${MHZ}mhz"
        ;;
    volume)
        # The same rate parameter as `verify`, because the point is the same:
        # the volume's addresses are checked at the rate the die can hold.
        MHZ="${VERIFY_MHZ:-40}"
        PLL="out/pipe_pll_${MHZ}.v"
        ROM="out/volume_rom.hex"
        if [ ! -s "$ROM" ]; then
            echo "missing $ROM: the volume's coordinate values are the input" >&2
            echo "run the host check that writes it, then build again" >&2
            exit 1
        fi
        COUNT=$(wc -l < "$ROM" | tr -d ' ')
        if [ "$COUNT" -eq 0 ]; then
            echo "$ROM is empty, so there is nothing to check" >&2
            exit 1
        fi
        echo "--- ecppll: 25 MHz to $MHZ MHz ---"
        ecppll -i 25 -o "$MHZ" -f "$PLL" --module pipe_pll \
            --clkin_name clk_in --clkout0_name clk >/dev/null
        SRC="../../rtl/tagma_decoder_pipe.v $PLL ../../rtl/demo/tagma_volume_verify_top.v"
        TOP=tagma_volume_verify_top
        LPF="$BOARDS/icesugar_pro/volume.lpf"
        # The host computes this over the same values, so the die is checked
        # against a second implementation rather than against itself.
        CHECKSUM="${EXPECTED_CHECKSUM:?set EXPECTED_CHECKSUM to the value the host computed}"
        PARAMS="chparam -set EXPECTED 32'h$CHECKSUM -set COUNT $COUNT $TOP"
        NAME="${TOP}_${MHZ}mhz"
        REPORT="volume_${MHZ}mhz"
        ;;
    led)
        SRC="../../rtl/demo/led_check_top.v"
        TOP=led_check_top
        LPF="$BOARDS/icesugar_pro/led.lpf"
        PARAMS=""
        ;;
    stage)
        SRC="../../rtl/demo/stage_check_top.v"
        TOP=stage_check_top
        LPF="$BOARDS/icesugar_pro/stage.lpf"
        # MIRROR selects which stage bit reaches pin_match, so a host can test
        # one input at a time without a person watching the LED.
        PARAMS="chparam -set MIRROR ${MIRROR:-0} $TOP"
        NAME="stage_check_top_m${MIRROR:-0}"
        REPORT="stage_m${MIRROR:-0}"
        ;;
    *)
        echo "unknown design for $BOARD: $DESIGN (expected bringup, tagma_demo, stream, stream_check, verify, volume, led or stage)" >&2
        exit 1
        ;;
    esac

    if [ -z "$NAME" ]; then
        NAME="$TOP"
    fi

    # No ECP5 equivalent of icebox_vlog exists, so this flow ends at the
    # bitstream and the nextpnr timing summary.
    echo "--- yosys: $DESIGN synthesis (ECP5) ---"
    YOSYS_SCRIPT="read_verilog -sv $SRC"
    if [ -n "$PARAMS" ]; then
        YOSYS_SCRIPT="$YOSYS_SCRIPT; $PARAMS"
    fi
    YOSYS_SCRIPT="$YOSYS_SCRIPT; synth_ecp5 -top $TOP -json out/$NAME.json"
    yosys -q -p "$YOSYS_SCRIPT"

    echo "--- nextpnr-ecp5: place and route ---"
    nextpnr-ecp5 --25k --package CABGA256 --speed 6 \
        --json "out/$NAME.json" \
        --lpf "$LPF" \
        --textcfg "out/$NAME.config" \
        --freq 25 2>&1 | tee "reports/pnr_${BOARD}_${REPORT}.log" | tail -20

    echo "--- ecppack: bitstream ---"
    ecppack "out/$NAME.config" "out/$NAME.bit"

    echo "bitstream: out/$NAME.bit"
    echo "config:    out/$NAME.config"
    echo "timing:    reports/pnr_${BOARD}_${REPORT}.log"
    ;;

*)
    echo "unknown board: $BOARD (expected upduino31 or icesugar_pro)" >&2
    exit 1
    ;;
esac
