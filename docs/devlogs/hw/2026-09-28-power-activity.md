# Power from measured switching activity (issue #70)

## Why

The ORFS flow reports 0.881 mW for the registered demo top at the 12 MHz board clock, measured with the current ORFS image. That number rests on the tool's default input activity, not on the workload the demo runs.

## What the tool does

Read from the OpenSTA source tree, and from the command's own help text in the image:

- `set_power_activity` takes `-global`, `-input`, `-input_ports`, `-pins`, `-activity`, `-density`, `-duty`, `-clock`, `-leakage`.
- `-activity` is documented as the number of transitions per clock cycle, with `-clock` choosing the clock whose period is used and the minimum-period clock as the default. `-density` is transitions per library time unit. `-duty` is the probability that the signal is high.
- The input default is `set_power_activity -input -activity 0.1 -duty 0.5`, so every input starts at 0.1 transitions per clock cycle.
- The help states that activities are propagated from annotated input ports or pins through gates and used in the power calculation. The model is vectorless: annotate the inputs, let the tool carry the activity inward.
- `read_vcd` and `read_saif` are the file-based route, which option B in issue #70 would use.
- A search of `OpenROAD-flow-scripts` for `set_power_activity` returns nothing. ORFS sets no activity anywhere, so the flow's power figure is the tool default plus whatever the clock forces.

The two activity options were tied together by measurement on the routed design, not by reading:

| command | total power |
|---------|-------------|
| `-input -activity 0.1` | 8.81e-04 W |
| `-input -density 1.200048e-3` | 8.81e-04 W |
| `-input -density 0.1` | 3.64e-02 W |
| `-input -activity 1.0` | 8.37e-03 W |

The first two agree to the last digit. The clock period is 83.33 ns, so 0.1 transitions per clock is 1.20005e6 transitions per second, and the liberty's `time_unit` is `1ns`, so the same rate is 1.20005e-3 transitions per nanosecond. The two options carry the same rate in different units, exactly as their help text says.

## What the environment allows

- The development host has no PDK and keeps no Liberty for the tools themselves, but the ORFS image is pullable and the flow runs inside it. On the arm64 host the image runs under emulation, with Docker warning that the requested platform is `linux/amd64` and the host is `linux/arm64/v8`. The tiny demo design is small enough that the whole flow completes this way.
- The ORFS image is built from `openroad/flow-dev` and carries yosys and openroad.
- The `sky130hd` platform ships `lib/`, `lef/`, `gds/`, and yosys techmap files for adders, clock gates, and latches. It ships no functional Verilog for the standard cells.
- Consequence: the Sky130 netlist that ORFS reports power on cannot be simulated from the flow alone, so no VCD of it can be produced. Option B therefore needs the open_pdks cell models and a simulator on top of the flow. Option A needs neither.
- Measured on the host: the RTL VCD carries clean names (`code`, `i`, `m`, `f`, `offset`, `q4`, `r2`), while the abc-mapped netlist keeps the port names and renames everything internal. Activity annotated by name would reach the ports alone, which is the second reason option A starts from the inputs.

## How the flow writes its results and reports power

Read from the ORFS sources and confirmed against a local run of the flow.

Results sit under a variant directory, `results/sky130hd/tagma_demo/base/`, which carries `1_2_yosys.v` (post-synthesis), `6_final.v`, `6_final.sdc`, `6_final.spef`, `6_final.def`, `6_final.gds`, and the per-step ODBs.

The final netlist is `module tagma_demo_top (clk, ...)` with `input [15:0] code`, so after the design is linked the code bits appear as `code[0]` through `code[15]`, which is what the hook annotates. `get_ports` returns all 35 ports, of which 16 are code bits.

The report step is a chain, and the design it reports on is not the last artifact it writes:

1. `final_connect.tcl` calls `erase_non_stage_variables final`, then `load_design 6_1_fill.odb 6_1_fill.sdc`, so the reported design comes from `6_1_fill.odb` with `6_1_fill.sdc`, not from `6_final`. It then calls `set_propagated_clock [all_clocks]` and `global_connect`.
2. `final_outputs.tcl` deletes routing obstructions, writes `6_final.def` and `6_final.v`, runs the RCX extraction branch (`extract_parasitics`, `write_spef`, `read_spef`), then calls `report_metrics 6 "finish"`.
3. `report_metrics` calls `report_power`, in a session where the liberty was read from `LIB_FILES` (the platform's `sky130_fd_sc_hd__tt_025C_1v80.lib`, a single corner because `sky130hd` sets no `CORNERS`), the derate and the platform `setRC.tcl` were sourced, and the SPEF is loaded.
4. `final_outputs.tcl` then calls `source_step_tcl POST FINAL_REPORT`, which sources `$::env(POST_FINAL_REPORT_TCL)`.

So the flow's 0.881 mW rests on the state after step 3. A separate script that reproduces it by hand has to reproduce `6_1_fill.odb`, `6_1_fill.sdc`, the propagated clock, the derate, the platform `setRC.tcl`, and the SPEF, which is a long list to get wrong.

## The first attempt and why it failed

The first version ran a separate `openroad` process that read the liberty and the final netlist and called `link_design`. In the OpenROAD shell `link_design` is OpenROAD's own procedure (`src/dbSta/src/dbReadVerilog.tcl`), and it raises `ORD-2010` when `ord::db_has_tech` is false:

```
[ERROR ORD-2010] no technology has been read.
    while executing
"utl::error ORD 2010 "no technology has been read.""
    (procedure "link_design" line 10)
    invoked from within
"link_design $top"
```

Reading LEF files would have satisfied it, and reading `6_final.odb` with `read_db` would have avoided `link_design` entirely. Neither removes the other problem: a separate process still has to reproduce the whole state listed above, and the two numbers would then rest on two different bases. The hook below runs in the flow's own session instead.

## The hook

The design config names the script as `POST_FINAL_REPORT_TCL`, so the flow sources it in the session that already holds the flow's basis:

```
export POST_FINAL_REPORT_TCL = $(DESIGN_HOME)/sky130hd/tagma_demo/power_activity.tcl
```

`POST_FINAL_REPORT_TCL` is declared in `flow/scripts/variables.yaml` with stage `final`, the same stage name `final_connect.tcl` passes to `erase_non_stage_variables`, so the variable is not erased before the hook runs. The single-process flow (`scripts/flow.tcl`) also sets `KEEP_VARS`, which disables the erase outright.

The hook loads nothing. It reports once at the tool default, which repeats the flow's own number, and once with the code bits annotated at the sweep's activity. With nothing set the inputs sit at the tool default activity 0.1 and duty 0.5, so the first report is the control: it shows the file is on the flow's basis, and the second report then differs only by the input activity.

The basis is the stimulus, not a saved trace. The demo sweeps all 65,536 code points, one per clock, so bit `b` of the counter changes value once every `2^b` clocks, an activity of `1 / 2^b` transitions per clock cycle. That is the unit `-activity` takes, so the annotation is `1 / 2^b` and the hook does not need the clock frequency at all.

The hook writes `$RESULTS_DIR/power_activity.rpt`, which the flow leaves in `results/sky130hd/tagma_demo/base/`, and `hw/openroad/run.sh` reads it out of the flow volume, so a hook that never ran, or that annotated only part of the code bits, fails the gate.

## Three defects, each found by running it

### The TCL format string

The first version printed each annotation with `format "  code[%d]: activity %.6g duty 0.5" $bit $activity`. TCL performs command substitution inside a double-quoted string, so `[%d]` expands before `format` runs and the interpreter looks for a command named `%d`, which does not exist. The error is `invalid command name "%d"`, and it sat on the success path, so the first annotated port would have ended the run after the first report. A local `tclsh` probe confirmed the two forms:

```
unescaped: ERROR: invalid command name "%d"
escaped: code[3]
```

The script writes `\[` and `\]`, which reach `format` literally. The same probe checked `double(1 << $bit)` against `pow(2, $bit)` and they agree, so the shift form is used.

### The iteration command that does not exist

The first run inside the flow failed with `invalid command name "foreach_in_collection"`. The port loop assumed `foreach_in_collection`, which this OpenROAD does not define; `info commands foreach_in_collection` returns empty, while `get_ports`, `get_name`, `get_full_name`, `get_property`, `set_power_activity`, and `report_power` all answer. `get_ports` returns a plain Tcl list of port objects in this build, so the loop is `foreach port [get_ports]` and the name comes from `get_name`.

The Tcl stack attributed the failure to the line that starts the loop, because the body of an iterating command has no line of its own in the file. That is why the trace pointed at the loop head and showed the body underneath it.

### The activity unit

The first working version passed `f / 2^b`, a per-second rate, to `-activity`, which reads its argument as transitions per clock cycle. The annotation was therefore larger than the stimulus by the clock frequency. This one is the quietest of the three: the annotated report still differed from the default, so the falsifier list would not have caught it. What it produced was 1.04e-01 W, close to the 1.05e-01 W the model reaches at `-density 1.200048e7`, so an activity far past any physical rate shows up as a saturated number rather than as an obvious error.

## Result

Measured by sourcing the hook against the routed design (`read_db 6_final.odb`, `read_sdc 6_final.sdc`, `set_propagated_clock [all_clocks]`, `read_spef 6_final.spef`) in the ORFS image:

| report | total power |
|--------|-------------|
| the flow's own `6_finish.rpt` | 8.81e-04 W |
| the hook at the tool default | 8.81e-04 W |
| the hook at the sweep's activity | 7.45e-05 W |

The hook's default report reproduces the flow's own number to the digit, which is the control that the two reports share a basis. At the demo's activity the total is about 12 times lower, not higher: the tool puts 0.1 transitions per clock on every input, while the sweep's upper code bits, which drive the subtraction and the divisions, change far less often than that. The tool's blank default overstates this design, and the direction was worth measuring rather than assuming.

The falsifier written before the run, that the total should rise above the default, was wrong in its premise. `1 / 2^b` is above 0.1 only for bits 0 to 2 and below it for the other thirteen.

## What would falsify this

- The two reports come out identical, which would mean the annotation did not take.
- The first report does not reproduce the flow's own number, which would mean the hook is not on the flow's basis and the pair would not be comparable. It did reproduce it.
- The applied activity does not scale as `1 / 2^b` across the code bits. Every annotation is printed, and they were 1, 0.5, 0.25, and down to 3.05176e-05 for bit 15.

## What was run and what was not

Run locally: `bash -n` on `hw/openroad/run.sh`; brace and paren balance on the hook; the `tclsh` probe for the format string; the activity-unit mapping in the ORFS image against the routed design; `bash hw/openroad/run.sh` end to end, in which the flow ran, the hook ran inside the flow's report step, and `results/power_activity.txt` was written from the file the hook left in the volume; and the hook sourced directly against the routed design for the table above.

Not run locally: the `hw` CI job with the corrected hook. That job is the gate.

## Open questions

- Whether a VCD of the routed netlist is worth its cost. It would measure the activity rather than model it from the stimulus, and the platform ships no functional Verilog for its cells, so it needs the open_pdks models plus a simulator. That is option B in issue #70.
- Whether the ceiling seen at extreme densities is a documented clamp or an artifact of the internal power model. It does not affect the hook, whose annotations stay at or below 1 transition per clock.
