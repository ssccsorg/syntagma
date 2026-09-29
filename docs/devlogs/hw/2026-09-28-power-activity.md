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

- Development host: no PDK, no Liberty anywhere on disk, no cached ORFS image. The Docker registry is reachable and 115 GiB is free. Everything about the ORFS path is settled by CI.
- The ORFS image is built from `openroad/flow-dev` and carries yosys and openroad.
- The `sky130hd` platform ships `lib/`, `lef/`, `gds/`, and yosys techmap files for adders, clock gates, and latches. It ships no functional Verilog for the standard cells.
- Consequence: the Sky130 netlist that ORFS reports power on cannot be simulated from the flow alone, so no VCD of it can be produced. Option B therefore needs the open_pdks cell models and a simulator on top of the flow. Option A needs neither.
- Measured on the host: the RTL VCD carries clean names (`code`, `i`, `m`, `f`, `offset`, `q4`, `r2`), while the abc-mapped netlist keeps the port names and renames everything internal. Activity annotated by name would reach the ports alone, which is the second reason option A starts from the inputs.

## How the flow writes its results and reports power

Read from the ORFS sources and from the `sky130-reports` artifact of a passing `hw` run.

Results sit under a variant directory, `results/sky130hd/tagma_demo/base/`, which carries `1_2_yosys.v` (post-synthesis), `6_final.v`, `6_final.sdc`, `6_final.spef`, `6_final.def`, `6_final.gds`, and the per-step ODBs.

The final netlist is `module tagma_demo_top (clk, ...)` with `input [15:0] code`, so after the design is linked the code bits appear as `code[0]` through `code[15]`, which is what the hook annotates.

The report step is a chain, and the design it reports on is not the last artifact it writes:

1. `final_connect.tcl` calls `erase_non_stage_variables final`, then `load_design 6_1_fill.odb 6_1_fill.sdc`, so the reported design comes from `6_1_fill.odb` with `6_1_fill.sdc`, not from `6_final`. It then calls `set_propagated_clock [all_clocks]` and `global_connect`.
2. `final_outputs.tcl` deletes routing obstructions, writes `6_final.def` and `6_final.v`, runs the RCX extraction branch (`extract_parasitics`, `write_spef`, `read_spef`), then calls `report_metrics 6 "finish"`.
3. `report_metrics` calls `report_power`, in a session where the liberty was read from `LIB_FILES` (the platform's `sky130_fd_sc_hd__tt_025C_1v80.lib`, a single corner because `sky130hd` sets no `CORNERS`), the derate and the platform `setRC.tcl` were sourced, and the SPEF is loaded.
4. `final_outputs.tcl` then calls `source_step_tcl POST FINAL_REPORT`, which sources `$::env(POST_FINAL_REPORT_TCL)`.

So the flow's 0.897 mW rests on the state after step 3. Any script that reproduces it by hand has to reproduce `6_1_fill.odb`, `6_1_fill.sdc`, the propagated clock, the derate, the platform `setRC.tcl`, and the SPEF, which is a long list to get wrong.

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

Reading LEF files would have satisfied it, and reading `6_final.odb` with `read_db` would have avoided `link_design` entirely. Neither removes the other problem: a separate process still has to reproduce the whole state listed above, and the two numbers would then rest on two different bases.

## The hook

The design config names the script as `POST_FINAL_REPORT_TCL`, so the flow sources it in the session that already holds the flow's basis:

```
export POST_FINAL_REPORT_TCL = $(DESIGN_HOME)/sky130hd/tagma_demo/power_activity.tcl
```

`POST_FINAL_REPORT_TCL` is declared in `flow/scripts/variables.yaml` with stage `final`, the same stage name `final_connect.tcl` passes to `erase_non_stage_variables`, so the variable is not erased before the hook runs. The single-process flow (`scripts/flow.tcl`) also sets `KEEP_VARS`, which disables the erase outright.

The hook loads nothing. It reports once at the tool default, which repeats the flow's own number, and once with the code bits annotated at the sweep's activity. With nothing set the inputs sit at the tool default activity 0.1 and duty 0.5, so the first report is the control: it shows the file is on the flow's basis, and the second report then differs only by the input activity.

The basis is the stimulus, not a saved trace. The demo sweeps all 65,536 code points, one per board clock of 83.33 ns. Bit `b` of a counter incremented once per clock changes value `65536 / 2^b` times per sweep of 65,536 clocks, so its transition rate is `f / 2^b` and its duty is 0.5.

The hook writes `$RESULTS_DIR/power_activity.rpt` and echoes the same narration to the flow log, and `hw/openroad/run.sh` reads the file out of the flow volume, so a hook that never ran, or that annotated only part of the code bits, fails the gate.

## A fatal TCL bug on the success path

The first version printed each annotation with `format "  code[%d]: activity %.6g duty 0.5" $bit $activity`. TCL performs command substitution inside a double-quoted string, so `[%d]` expands before `format` runs and the interpreter looks for a command named `%d`, which does not exist. The error is `invalid command name "%d"`, and it sat on the success path, so the first annotated port would have ended the run after the first report. A local `tclsh` probe confirmed the two forms:

```
unescaped: ERROR: invalid command name "%d"
escaped: code[3]
```

The script writes `\[` and `\]`, which reach `format` literally.

The same probe checked `double(1 << $bit)` against `pow(2, $bit)` and they agree, so the shift form is used. It also showed that `get_name` is absent from a bare `tclsh`, so the name is read through the first accessor that answers, under `catch`, and the step reports the ports it could not name rather than annotating a subset.

## What would falsify this

- The two reports come out identical, which would mean the annotation did not take.
- The first report does not reproduce the flow's own number, which would mean the hook is not on the flow's basis and the pair would not be comparable.
- The applied activity does not scale as `1 / 2^b` across the code bits.
- The total does not rise above the default once the data path starts switching, which would mean the propagation assumption is wrong.

## What was run and what was not

Run locally: `bash -n` on `hw/openroad/run.sh`, TCL brace and paren balance on the hook, and the `tclsh` probe above. The RTL gate (`make -C hw check`, `make -C hw sim-pnr`) is unchanged by this step.

Not run locally: the ORFS flow and the hook. The host has no PDK and no cached image, so CI is the only place the hook executes. The redirection in `report_power >> $rpt` is the same idiom `report_metrics` uses one call earlier in the same session, which is why it is trusted here without a local run.

## Open questions

- The exact role of `activity` in the switching formula. The help text says transitions per second; a factor of two against the textbook `alpha * C * V^2 * f` is possible, and the two-report comparison is what will show it.
- Which of `get_name`, `get_full_name`, or `get_property ... name` the OpenROAD build provides, and whether the returned name escapes the brackets. The script tries them in order under `catch`, accepts either form in the regexp, and reports how many ports it could not name.
- Whether `-input_ports` accepts the collection object `foreach_in_collection` yields rather than a list of names.
- Whether `report_power >> $rpt` appends as expected. It is the flow's own redirection idiom, but this script is the first in `hw/` to use it.
