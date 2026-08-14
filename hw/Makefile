# Tagma hardware verification targets.
#
# Mirrors the Makefile pattern in ssccs/poc/baremetal_riscv/sv:
# simulation with Verilator, synthesis with Yosys, reports as artifacts.
#
# Usage:
#   make sim            # exhaustive Verilator simulation (formula mode)
#   make sim-golden     # Verilator simulation against golden anchors
#   make golden-export  # regenerate golden anchors from tagma_core
#   make golden-check   # Python consistency gate on the golden file
#   make synth          # Yosys generic, gate-level, and iCE40 reports
#   make check          # sim + sim-golden + golden-check + synth
#   make clean

SIM   ?= verilator
YOSYS ?= yosys

RTL = rtl/tagma_decoder.v
TB  = rtl/tagma_decoder_tb.v

OBJ_DIR = obj_dir
GOLDEN  = rtl/golden_anchors.hex

.PHONY: all sim sim-golden golden-export golden-check synth report check clean

all: check

sim: $(RTL) $(TB)
	$(SIM) --binary --timing -Wall --top-module tagma_decoder_tb $(TB) $(RTL) -o tb_sim
	./$(OBJ_DIR)/tb_sim

sim-golden: golden-export $(RTL) $(TB)
	$(SIM) --binary --timing -Wall -DGOLDEN_CHECK --top-module tagma_decoder_tb $(TB) $(RTL) -o tb_golden
	cd rtl && ../$(OBJ_DIR)/tb_golden

golden-export:
	cd ../sw/rust && cargo run --release -p tagma-core --example golden_export -- ../../hw/rtl/golden_anchors.hex

golden-check:
	python3 tools/check_golden_anchors.py

synth:
	cd synth/yosys && ./run.sh

report: synth
	@echo "=== gate-level estimate ==="
	@cat synth/yosys/reports/gates_stat.txt
	@echo "=== iCE40 ==="
	@cat synth/yosys/reports/ice40_stat.txt

check: sim sim-golden golden-check synth

clean:
	rm -rf $(OBJ_DIR) synth/yosys/out synth/yosys/reports $(GOLDEN)
