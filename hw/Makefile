# Tagma hardware verification targets.
#
# Mirrors the Makefile pattern in ssccs/poc/baremetal_riscv/sv:
# simulation with Verilator, synthesis with Yosys, reports as artifacts.
#
# Usage:
#   make sim            # exhaustive Verilator simulation (formula mode)
#   make sim-golden     # Verilator simulation against golden anchors
#   make sim-trace      # golden simulation with VCD activity trace
#   make golden-export  # regenerate golden anchors from tagma_core
#   make golden-check   # Python consistency gate on the golden file
#   make gatesim        # gate-level netlist simulation against golden anchors
#   make equiv          # formal equivalence: RTL vs gate netlist
#   make synth          # Yosys generic, gate-level, and iCE40 reports
#   make check          # sim + sim-golden + golden-check + gatesim + equiv + synth
#   make clean

SIM   ?= verilator
YOSYS ?= yosys

RTL = rtl/tagma_decoder.v
TB  = rtl/tagma_decoder_tb.v

OBJ_DIR = obj_dir
GOLDEN  = rtl/golden_anchors.hex

.PHONY: all sim sim-golden sim-trace golden-export golden-check gatesim equiv synth report check clean

all: check

sim: $(RTL) $(TB)
	$(SIM) --binary --timing -Wall --top-module tagma_decoder_tb $(TB) $(RTL) -o tb_sim
	./$(OBJ_DIR)/tb_sim

sim-golden: golden-export $(RTL) $(TB)
	$(SIM) --binary --timing -Wall -DGOLDEN_CHECK --top-module tagma_decoder_tb $(TB) $(RTL) -o tb_golden
	cd rtl && ../$(OBJ_DIR)/tb_golden

sim-trace: $(RTL) $(TB)
	$(SIM) --binary --timing -Wall -DTRACE --trace --top-module tagma_decoder_tb $(TB) $(RTL) -o tb_trace
	cd rtl && ../$(OBJ_DIR)/tb_trace

golden-export:
	cd ../sw/rust && cargo run --release -p tagma-core --example golden_export -- ../../hw/rtl/golden_anchors.hex

golden-check:
	python3 tools/check_golden_anchors.py

gatesim: golden-export synth
	$(SIM) --binary --timing -Wall -Wno-UNUSEDSIGNAL -Wno-UNDRIVEN -Wno-TIMESCALEMOD -Wno-DECLFILENAME -DGOLDEN_CHECK --top-module tagma_decoder_tb synth/yosys/out/tagma_decoder_gates.v $(TB) -o tb_gatesim
	cd rtl && ../$(OBJ_DIR)/tb_gatesim

equiv: synth
	cd synth/yosys && yosys -q -s equiv.ys && echo "equiv: RTL and gate netlist are formally equivalent"

synth:
	cd synth/yosys && ./run.sh

report: synth
	@echo "=== gate-level estimate ==="
	@cat synth/yosys/reports/gates_stat.txt
	@echo "=== iCE40 ==="
	@cat synth/yosys/reports/ice40_stat.txt

check: sim sim-golden golden-check gatesim equiv synth

clean:
	rm -rf $(OBJ_DIR) synth/yosys/out synth/yosys/reports $(GOLDEN) rtl/trace.vcd
