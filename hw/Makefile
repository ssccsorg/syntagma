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
# Tagma hardware verification targets.
#
# Mirrors the Makefile pattern in ssccs/poc/baremetal_riscv/sv:
# simulation with Verilator, synthesis with Yosys, reports as artifacts.
#
# Usage:
#   make sim            # exhaustive Verilator simulation (formula mode)
#   make sim-golden     # Verilator simulation against golden anchors
#   make sim-trace      # golden simulation with VCD activity trace
#   make sim-store      # segment store read/write over all 11,172 slots
#   make golden-export  # regenerate golden anchors from tagma_core
#   make golden-check   # Python consistency gate on the golden file
#   make gatesim        # gate-level netlist simulation against golden anchors
#   make equiv          # formal equivalence: RTL vs gate netlist
#   make synth          # Yosys generic, gate-level, and iCE40 reports
#   make check          # sim + sim-golden + sim-store + golden-check + gatesim + equiv + synth
#   make clean

SIM   ?= verilator
YOSYS ?= yosys

RTL = rtl/tagma_decoder.v
TB  = rtl/tagma_decoder_tb.v

STORE_RTL = rtl/tagma_segment_store.v
STORE_TB  = rtl/tagma_segment_store_tb.v

OBJ_DIR = obj_dir
GOLDEN  = rtl/golden_anchors.hex

# Verilator build output is split: the compiler command spam (stdout) goes
# to a per-target log in the object dir, warnings and errors (stderr) stay
# visible, the version/walltime tail is echoed on success, and the full log
# is dumped if the build fails. The simulation run itself always prints its
# PASS/FAIL line.
VERILATE = $(SIM) --binary --timing -Wall --top-module tagma_decoder_tb
VERILATE_STORE = $(SIM) --binary --timing -Wall --top-module tagma_segment_store_tb

# Verilator redirects its build log into the object dir, which is a
# generated (gitignored) directory: ensure it exists before any sim target
# runs, including in a fresh checkout or Docker build.
$(OBJ_DIR):
	mkdir -p $@

.PHONY: all sim sim-golden sim-trace sim-store golden-export golden-check gatesim equiv synth coverage report check clean

all: check

sim: $(RTL) $(TB) | $(OBJ_DIR)
	@echo "--- verilator: tb_sim (formula mode) ---"
	@$(VERILATE) $(TB) $(RTL) -o tb_sim \
		> $(OBJ_DIR)/build_tb_sim.log 2> $(OBJ_DIR)/build_tb_sim.err \
		|| { echo "verilator build failed:"; cat $(OBJ_DIR)/build_tb_sim.log $(OBJ_DIR)/build_tb_sim.err; exit 1; }
	@if test -s $(OBJ_DIR)/build_tb_sim.err; then echo "verilator warnings:"; cat $(OBJ_DIR)/build_tb_sim.err; fi
	@tail -3 $(OBJ_DIR)/build_tb_sim.log
	@rm -f $(OBJ_DIR)/build_tb_sim.err
	@./$(OBJ_DIR)/tb_sim

sim-golden: golden-export $(RTL) $(TB) | $(OBJ_DIR)
	@echo "--- verilator: tb_golden (golden anchors) ---"
	@$(VERILATE) -DGOLDEN_CHECK $(TB) $(RTL) -o tb_golden \
		> $(OBJ_DIR)/build_tb_golden.log 2> $(OBJ_DIR)/build_tb_golden.err \
		|| { echo "verilator build failed:"; cat $(OBJ_DIR)/build_tb_golden.log $(OBJ_DIR)/build_tb_golden.err; exit 1; }
	@if test -s $(OBJ_DIR)/build_tb_golden.err; then echo "verilator warnings:"; cat $(OBJ_DIR)/build_tb_golden.err; fi
	@tail -3 $(OBJ_DIR)/build_tb_golden.log
	@rm -f $(OBJ_DIR)/build_tb_golden.err
	@cd rtl && ../$(OBJ_DIR)/tb_golden

sim-trace: $(RTL) $(TB) | $(OBJ_DIR)
	@echo "--- verilator: tb_trace (VCD activity trace) ---"
	@$(VERILATE) -DTRACE --trace $(TB) $(RTL) -o tb_trace \
		> $(OBJ_DIR)/build_tb_trace.log 2> $(OBJ_DIR)/build_tb_trace.err \
		|| { echo "verilator build failed:"; cat $(OBJ_DIR)/build_tb_trace.log $(OBJ_DIR)/build_tb_trace.err; exit 1; }
	@if test -s $(OBJ_DIR)/build_tb_trace.err; then echo "verilator warnings:"; cat $(OBJ_DIR)/build_tb_trace.err; fi
	@tail -3 $(OBJ_DIR)/build_tb_trace.log
	@rm -f $(OBJ_DIR)/build_tb_trace.err
	@cd rtl && ../$(OBJ_DIR)/tb_trace

sim-store: $(STORE_RTL) $(STORE_TB) | $(OBJ_DIR)
	@echo "--- verilator: tb_store (segment store) ---"
	@$(VERILATE_STORE) $(STORE_TB) $(STORE_RTL) -o tb_store \
		> $(OBJ_DIR)/build_tb_store.log 2> $(OBJ_DIR)/build_tb_store.err \
		|| { echo "verilator build failed:"; cat $(OBJ_DIR)/build_tb_store.log $(OBJ_DIR)/build_tb_store.err; exit 1; }
	@if test -s $(OBJ_DIR)/build_tb_store.err; then echo "verilator warnings:"; cat $(OBJ_DIR)/build_tb_store.err; fi
	@tail -3 $(OBJ_DIR)/build_tb_store.log
	@rm -f $(OBJ_DIR)/build_tb_store.err
	@./$(OBJ_DIR)/tb_store

golden-export:
	@echo "--- golden export: tagma_core -> golden_anchors.hex ---"
	@cd ../sw/rust && cargo run --quiet --release -p tagma-core --example golden_export -- ../../hw/rtl/golden_anchors.hex

golden-check:
	python3 tools/check_golden_anchors.py

gatesim: golden-export synth | $(OBJ_DIR)
	@echo "--- verilator: tb_gatesim (gate netlist) ---"
	@$(VERILATE) -Wno-UNUSEDSIGNAL -Wno-UNDRIVEN -Wno-TIMESCALEMOD -Wno-DECLFILENAME -DGOLDEN_CHECK synth/yosys/out/tagma_decoder_gates.v $(TB) -o tb_gatesim \
		> $(OBJ_DIR)/build_tb_gatesim.log 2> $(OBJ_DIR)/build_tb_gatesim.err \
		|| { echo "verilator build failed:"; cat $(OBJ_DIR)/build_tb_gatesim.log $(OBJ_DIR)/build_tb_gatesim.err; exit 1; }
	@if test -s $(OBJ_DIR)/build_tb_gatesim.err; then echo "verilator warnings:"; cat $(OBJ_DIR)/build_tb_gatesim.err; fi
	@tail -3 $(OBJ_DIR)/build_tb_gatesim.log
	@rm -f $(OBJ_DIR)/build_tb_gatesim.err
	@cd rtl && ../$(OBJ_DIR)/tb_gatesim

equiv: synth
	cd synth/yosys && yosys -q -s equiv.ys && echo "equiv: RTL and gate netlist are formally equivalent"

synth:
	cd synth/yosys && bash run.sh

report: synth
	@echo "=== gate-level estimate ==="
	@cat synth/yosys/reports/gates_stat.txt
	@echo "=== iCE40 ==="
	@cat synth/yosys/reports/ice40_stat.txt

check: sim sim-golden sim-store golden-check gatesim equiv synth

coverage: sim sim-golden sim-store golden-check gatesim equiv
	@echo "coverage: 11,172 / 11,172 valid code points in 3 channels (formula, golden, netlist) + 2^16 inputs in formal equivalence"
	@echo "coverage: the valid input space is exhaustively covered; invalid states are structurally detectable by design"
	@echo "coverage: 11,172 / 11,172 segment store slots read back, reserved addresses read zero"

clean:
	rm -rf $(OBJ_DIR) synth/yosys/out synth/yosys/reports $(GOLDEN) rtl/trace.vcd
