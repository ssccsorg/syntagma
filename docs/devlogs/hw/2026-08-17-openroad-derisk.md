# OpenROAD Sky130 flow derisking for the hw CI job

Status report for issue #48 on branch `48-hw-verification`. This entry records the local verification of the ORFS (OpenROAD Flow Scripts) setup for the hw CI job, and the fixes that came out of it. Two flow bugs were found and fixed: a virtual clock definition that left the design without a clock tree, and a default-on LEC step whose bundled tool crashes on the available CPUs. After the fixes the full flow runs end to end, including locally on Apple Silicon under Rosetta.

## ORFS layout correction

The design config originally used a nonstandard layout (`DESIGN_HOME` pointed at committed RTL copies). ORFS resolves `DESIGN_HOME ?= $(FLOW_HOME)/designs` (from `scripts/variables.mk`), so the standard layout, identical to the shipped `sky130hd/gcd` example, is:

```makefile
export VERILOG_FILES = $(DESIGN_HOME)/src/tagma_demo/tagma_decoder.v \
                       $(DESIGN_HOME)/src/tagma_demo/tagma_demo_top.v
export SDC_FILE      = $(DESIGN_HOME)/sky130hd/tagma_demo/constraint.sdc
```

`hw/openroad/run.sh` mounts the live RTL from `hw/rtl` onto `flow/designs/src/tagma_demo` and the config directory onto `flow/designs`, so the run always uses the current RTL with no committed copies to drift. The old `constraints.sdc` (plural) was replaced by `constraint.sdc` (singular), matching the ORFS convention.

The SDC follows the shipped examples: `set_input_delay` applies to `[all_inputs -no_clocks]` so the clock port is not treated as a data input.

## Virtual clock: CTS found no clock nets

The first constraint file created the clock without a source port:

```tcl
create_clock -period 83.33 -name clk
```

Without a source object this is a virtual clock. Timing analysis accepts it, but TritonCTS reports `No clock nets have been found` and builds no tree, which surfaced as a downstream crash. The fix follows the shipped examples:

```tcl
create_clock -period 83.33 -name clk [get_ports clk]
```

After the fix the CTS log shows `Net "clk" found for clock "clk"` with 16 sinks and a built H-tree.

## The kepler-formal LEC crash

The CTS stage still aborted after repair timing with `Error: cts.tcl, 83 child killed: illegal instruction`. The same error appeared under Rosetta and on the x86 CI runner, so it was not an emulation issue. Wrapping the stage in a Tcl catch exposed the full stack: the flow runs a logic equivalence check (LEC) after repair timing, and the bundled `kepler-formal` binary dies with SIGILL:

```text
child killed: illegal instruction
    while executing
"exec /OpenROAD-flow-scripts/tools/install/kepler-formal/bin/kepler-formal --config ./objects/sky130hd/tagma_demo/base/4_rsz_lec_test.yml"
    invoked from within
"run_lec_test 4_rsz 4_before_rsz_lec.v 4_after_rsz_lec.v"
    (file "cts.tcl" line 71)
```

ORFS enables LEC by default when the image ships kepler-formal (`settings.mk`: `LEC_CHECK ?= $(if $(wildcard $(KEPLER_FORMAL_EXE)),1,0)`). The binary crashes on the CPUs available here (Apple Silicon Rosetta and the GitHub Actions x86 runner). The tagma config disables the in-flow LEC check, since the RTL-to-netlist equivalence is already proven by the yosys equiv gate in `hw/`:

```makefile
export LEC_CHECK = 0
```

## The yosys abc extraction gap

The decoder gate count step in `run.sh` used `proc; opt` before `abc -liberty`. On the standalone yosys binary in the ORFS image (0.67), that extracts nothing: the multiplier cells stay as `$mul` and the abc pass reports `Extracted 0 gates` and `Don't call ABC as there is nothing to map`. The reproduction is a plain `a * b` module under `read_verilog; hierarchy; proc; opt; read_liberty; abc -liberty`, which also extracts 0 gates, while the same module through `synth` maps normally.

Fix: run `synth -top tagma_decoder` before `read_liberty; abc -liberty; stat -liberty`. The Sky130 result for the pure decoder is:

| Metric | Value |
|--------|-------|
| sky130hd cells | 388 |
| chip area | 2826.46 um^2 |
| registers | 0, combinational |

The 388 cell count is the abc mapping of the multiply-shift network against `sky130_fd_sc_hd__tt_025C_1v80.lib`, consistent with the 478 generic cells and the 588 gate-level estimate already reported.

## Floorplan initialization

The first floorplan attempt failed with `Error: No floorplan initialization method specified`. ORFS requires an explicit floorplan variable; the shipped configs use `CORE_UTILIZATION` (plus `CORE_ASPECT_RATIO` and `CORE_MARGIN` where needed). The tagma config now sets:

```makefile
export CORE_UTILIZATION = 30
export CORE_ASPECT_RATIO = 1
export CORE_MARGIN = 2
```

## Full flow result

With the clock and LEC fixes, the complete ORFS flow runs end to end: synth, floorplan, place, cts, route, finish. The run also completes on Apple Silicon under Rosetta; no x86-only stage remains. Measured for the registered demo top (12 MHz board-equivalent clock, 83.33 ns):

| Metric | Value |
|--------|-------|
| design area | 4631 um^2, 35% utilization |
| critical path delay | 11.19 ns (code[11] to m[4] register) |
| worst setup slack | +72.33 ns |
| total power | 0.897 mW (sequential 3.3%, combinational 95.5%, clock 1.2%) |
| pure decoder area | 2826.46 um^2, 388 cells (yosys stat -liberty) |

## PDK availability

The ORFS image ships the sky130hd platform files including the liberty at `platforms/sky130hd/lib/sky130_fd_sc_hd__tt_025C_1v80.lib`, and the flow Makefile has no pdk download target. A fresh volume run completes the flow without network access.

## Reproduction

```bash
docker volume rm tagma_orfs_flow   # optional, fresh start
bash hw/openroad/run.sh            # full flow; runs on x86, Apple Silicon, or the hw CI job
```
