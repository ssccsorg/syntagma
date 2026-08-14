# Tagma hardware verification targets.
#
# Mirrors the Makefile pattern in ssccs/poc/baremetal_riscv/sv:
# simulation with Verilator, synthesis with Yosys, reports as artifacts.
#
# Usage:
#   make sim      # exhaustive Verilator simulation
#   make synth    # Yosys generic, gate-level, and iCE40 reports
#   make check    # sim + synth
#   make clean

SIM   ?= verilator
YOSYS ?= yosys

RTL = rtl/tagma_decoder.v
TB  = rtl/tagma_decoder_tb.v

OBJ_DIR = obj_dir

.PHONY: all sim synth report check clean

all: check

sim: $(RTL) $(TB)
	$(SIM) --binary --timing -Wall --top-module tagma_decoder_tb $(TB) $(RTL) -o tb_sim
	./$(OBJ_DIR)/tb_sim

synth:
	cd synth/yosys && ./run.sh

report: synth
	@echo "=== gate-level estimate ==="
	@cat synth/yosys/reports/gates_stat.txt
	@echo "=== iCE40 ==="
	@cat synth/yosys/reports/ice40_stat.txt

check: sim synth

clean:
	rm -rf $(OBJ_DIR) synth/yosys/out synth/yosys/reports
