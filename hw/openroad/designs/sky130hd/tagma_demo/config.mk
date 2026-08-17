# ORFS design configuration for the Tagma decoder demo top.
#
# Target: sky130hd (SkyWater 130nm high-density standard cells).
# The registered demo top is used so the standard cell flow produces an
# Fmax and timing closure report that maps one to one to the FPGA demo.
# The pure decoder area is reported separately by yosys stat -liberty in
# run.sh.

export DESIGN_NICKNAME = tagma_demo
export DESIGN_NAME = tagma_demo_top
export PLATFORM    = sky130hd

export VERILOG_FILES = $(DESIGN_HOME)/src/tagma_decoder.v \
                       $(DESIGN_HOME)/src/tagma_demo_top.v
export SDC_FILE      = $(DESIGN_HOME)/constraints.sdc
export CLOCK_PERIOD  = 83.33
