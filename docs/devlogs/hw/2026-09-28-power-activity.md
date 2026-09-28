# Power from measured switching activity (issue #70)

## Why

The ORFS flow reports 0.897 mW for the registered demo top at the 12 MHz board clock. That number rests on the tool defaults, not on the workload the demo runs.

## What the tool does

Read from the OpenSTA source tree (`power/Power.tcl`, `power/test/power_report.tcl`, `doc/Commands.md`):

- `set_power_activity` takes `-global`, `-input`, `-input_ports`, `-pins`, `-activity`, `-duty`, `-leakage`. The input default is activity 0.1 and duty 0.5.
- The command's own help says: "The activities are propagated from annotated input ports or pins through gates and used in the power calculations." The model is therefore the standard vectorless one: annotate the inputs, let the tool carry the activity inward.
- The activity property is documented as "transitions per second", with duty alongside it, and the origin recorded (`global`, `input`, `user`, `vcd`, `saif`, `propagated`, `clock`, `constant`).
- `read_vcd` and `read_saif` are the file-based route, and they are what option B in issue #70 would use.
- A search of `OpenROAD-flow-scripts` for `set_power_activity` returns nothing. ORFS sets no activity anywhere, so the 0.897 mW figure is the tool default input activity (0.1, duty 0.5) plus whatever the clock forces. In other words the clock tree switches and the data path largely does not.

## What the environment allows

- Development host: no PDK, no Liberty anywhere on disk, no cached ORFS image. The Docker registry is reachable (`docker manifest inspect openroad/orfs:latest` returns the index) and 115 GiB is free.
- The ORFS image is built from `openroad/flow-dev` and carries yosys and openroad.
- The `sky130hd` platform ships `lib/`, `lef/`, `gds/`, and yosys techmap files for adders, clock gates, and latches. It ships no functional Verilog for the standard cells.
- Consequence: the Sky130 netlist that ORFS reports power on cannot be simulated from the flow alone, so no VCD of it can be produced. Option B therefore needs the open_pdks cell models and a simulator on top of the flow. Option A needs neither.
- Measured on the host: the RTL VCD carries clean names (`code`, `i`, `m`, `f`, `offset`, `q4`, `r2`), while the abc-mapped netlist keeps the port names and renames everything internal. Activity annotated by name would reach the ports alone, which is the second reason option A starts from the inputs.

## Where the flow writes

Read from the `sky130-reports` artifact of a passing `hw` run. The results sit under a variant directory, `results/sky130hd/tagma_demo/base/`, which carries `1_2_yosys.v` (post-synthesis), `6_final.v`, `6_final.sdc`, `6_final.spef`, `6_final.def`, `6_final.gds`, and the per-step ODBs.

The final netlist is `module tagma_demo_top (clk, ...)` with `input [15:0] code`, so after `link_design` the code bits appear as `code[0]` through `code[15]`, which is what the script annotates.

The step first assumed `results/sky130hd/tagma_demo/6_final.v`, a path that does not exist. The run failed on `ls: cannot access ...` and nothing else was attempted, so that failure is evidence about the path alone. The step now locates its inputs by name under the results tree and prints the tree it searched.

## Plan for this step

1. `hw/openroad/power_activity.tcl`: read the final netlist, the liberty, the final SDC, and the SPEF; propagate the clock; report power twice, once at the tool default input activity and once at the sweep's.
2. State the activity basis in the report. The demo sweeps all 65,536 code points, one per board clock of 83.33 ns. Bit `b` of a counter incremented once per clock changes value `65536 / 2^b` times per sweep of 65,536 clocks, so its transition rate is `f / 2^b` and its duty is 0.5. No VCD is needed for this step because the activity follows from the stimulus the testbench drives.
3. Wire the step into `hw/openroad/run.sh` after the flow, saving `results/power_activity.txt`.
4. Leave option B open in issue #70. It is the only route to a VCD-driven number.

## What would falsify this

- The two reports come out identical, which would mean the annotation did not take.
- The applied activity does not scale as `1 / 2^b` across the code bits.
- The total does not rise above the default once the data path starts switching, which would mean the propagation assumption is wrong.

## Open questions

- The exact role of `activity` in the switching formula. The help text says transitions per second; a factor of two against the textbook `alpha * C * V^2 * f` is possible, and the two-report comparison is what will show it.
- Whether `read_spef` needs a corner argument in this build, and whether the SPEF is emitted as `6_final.spef` by the `6_report` step.
- Whether `get_ports` matches `code[0]` without escaping in this build.
