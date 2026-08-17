# ORFS design configuration for the Tagma decoder demo top.
#
# Target: sky130hd (SkyWater 130nm high-density standard cells).
# The registered demo top is used so the standard cell flow produces an
# Fmax and timing closure report that maps one to one to the FPGA demo.
# The pure decoder area is reported separately by yosys stat -liberty in
# run.sh.
#
# ORFS layout: RTL lives under $(DESIGN_HOME)/src/<DESIGN_NICKNAME>/
# (mounted from hw/rtl by run.sh), and the config under
# $(DESIGN_HOME)/<PLATFORM>/<DESIGN_NICKNAME>/.

export DESIGN_NICKNAME = tagma_demo
export DESIGN_NAME = tagma_demo_top
export PLATFORM    = sky130hd

export VERILOG_FILES = $(DESIGN_HOME)/src/tagma_demo/tagma_decoder.v \
                       $(DESIGN_HOME)/src/tagma_demo/tagma_demo_top.v
export SDC_FILE      = $(DESIGN_HOME)/sky130hd/tagma_demo/constraint.sdc

export CLOCK_PERIOD  = 83.33

# Floorplan: modest utilization for the small registered demo top so the
# standard cell flow has room for routing (374 instances after synth).
export CORE_UTILIZATION = 30
export CORE_ASPECT_RATIO = 1
export CORE_MARGIN = 2

# LEC is enabled by default when the ORFS image ships kepler-formal, which
# crashes with an illegal instruction on the available CPUs (Apple Silicon
# Rosetta and the x86 CI runner). The RTL-to-netlist equivalence is already
# proven by the yosys equiv gate in hw/, so the in-flow LEC check is
# redundant here and is disabled.
export LEC_CHECK = 0
