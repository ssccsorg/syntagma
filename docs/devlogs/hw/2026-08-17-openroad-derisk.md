# OpenROAD Sky130 flow derisking for the p4 CI job

Status report for issue #48 on branch `48-hw-verification`. This entry records the local verification of the ORFS (OpenROAD Flow Scripts) setup before the first native run on the x86 p4 CI job, and the fixes that came out of it. The full flow cannot run on Apple Silicon: the OpenROAD binary crashes with an illegal instruction in the CTS step under Rosetta, so everything up to placement was exercised locally and the remaining stages are delegated to the x86 runner.

## ORFS layout correction

The design config originally used a nonstandard layout (`DESIGN_HOME` pointed at committed RTL copies). ORFS resolves `DESIGN_HOME ?= $(FLOW_HOME)/designs` (from `scripts/variables.mk`), so the standard layout, identical to the shipped `sky130hd/gcd` example, is:

```makefile
export VERILOG_FILES = $(DESIGN_HOME)/src/tagma_demo/tagma_decoder.v \
                       $(DESIGN_HOME)/src/tagma_demo/tagma_demo_top.v
export SDC_FILE      = $(DESIGN_HOME)/sky130hd/tagma_demo/constraint.sdc
```

`hw/openroad/run.sh` mounts the live RTL from `hw/rtl` onto `flow/designs/src/tagma_demo` and the config directory onto `flow/designs`, so the run always uses the current RTL with no committed copies to drift. The old `constraints.sdc` (plural) was replaced by `constraint.sdc` (singular), matching the ORFS convention.

The SDC follows the shipped examples: `set_input_delay` applies to `[all_inputs -no_clocks]` so the clock port is not treated as a data input.

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

## Local verification boundary

Stages verified locally under Rosetta with a fresh volume (exactly what CI starts from):

| Stage | Result |
|-------|--------|
| synth (`1_synth`) | pass, design area 4001 um^2 after synthesis |
| floorplan (`2_floorplan` incl. PDN) | pass, 4222 um^2, 32% utilization |
| place (`3_place` detailed) | pass, 4552 um^2, 35% utilization, 0 violations |
| cts (`4_cts`) | crash, `child killed: illegal instruction`, the known Rosetta limit |
| route and finish | not attempted locally, x86 only |

## PDK availability

The ORFS image ships the sky130hd platform files including the liberty at `platforms/sky130hd/lib/sky130_fd_sc_hd__tt_025C_1v80.lib`, and the flow Makefile has no pdk download target. A fresh volume run completes synthesis without network access, so the earlier p4 risk of a PDK auto-fetch is cleared: the only remaining p4-specific risk is the native execution of the CTS onward stages on the x86 runner.

## Reproduction

```bash
docker volume rm tagma_orfs_flow   # optional, fresh start
bash hw/openroad/run.sh            # full flow; run on x86 hardware or the p4 CI job
```
