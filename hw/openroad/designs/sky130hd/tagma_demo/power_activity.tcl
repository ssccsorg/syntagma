# SPDX-License-Identifier: CERN-OHL-P-2.0
# Copyright (C) 2026 Taeho Lee
#
# This source describes Open Hardware and is licensed under the CERN-OHL-P v2.
# You may redistribute and modify this documentation and make products using
# it under the terms of the CERN-OHL-P v2 (https://cern.ch/cern-ohl). This
# documentation is distributed WITHOUT ANY EXPRESS OR IMPLIED WARRANTY,
# INCLUDING OF MERCHANTABILITY, SATISFACTORY QUALITY AND FITNESS FOR A
# PARTICULAR PURPOSE. Please see the CERN-OHL-P v2 for applicable conditions.
#
# Power for the Tagma demo at the workload's input activity (issue #70).
#
# The design config names this file as POST_FINAL_REPORT_TCL, so the flow
# sources it at the end of its report step, after the design, the liberty, the
# sdc, the derate, the setRC, and the SPEF are in place. Nothing is loaded here:
# staying in the flow's own session is what keeps the report on the flow's
# basis, so the two reports below differ only by the input activity.
#
# OpenSTA computes switching power from per-pin activity, and with nothing set
# the inputs sit at the tool default of 0.1 transitions per clock cycle. The
# first report repeats the number the flow reports, which is the check that this
# file is on the flow's basis. The second annotates the primary inputs with the
# activity the demo presents; OpenSTA propagates it inward, so the data path is
# counted too.
#
# The basis is the stimulus, not a saved trace. The demo sweeps all 65,536 code
# points, one per clock, so bit b of the counter changes value once every 2^b
# clocks, which is an activity of 1 / 2^b transitions per clock cycle.
# set_power_activity's -activity is in that unit; -density is the per-time-unit
# form and would need the clock frequency.
#
# This build defines no foreach_in_collection, and get_ports returns a plain Tcl
# list of port objects, so the port loop is a plain foreach and names come from
# get_name.
#
# Output: $RESULTS_DIR/power_activity.rpt, which hw/openroad/run.sh reads out.

set power_rpt [file join $::env(RESULTS_DIR) power_activity.rpt]
close [open $power_rpt w]

# Append a line to the report and echo it, so the flow log carries the same
# narration as the file. The flow's report_metrics.tcl uses the same reopen
# pattern for its own report; report_power itself is appended with >>.
proc power_report_put { line } {
    set f [open $::env(RESULTS_DIR)/power_activity.rpt a]
    puts $f $line
    close $f
    puts $line
}

power_report_put "power at the demo workload's input activity (issue #70)"
power_report_put ""
power_report_put "=== at the tool default input activity (0.1 per clock, duty 0.5), the basis the flow reports on ==="
set_power_activity -input -activity 0.1 -duty 0.5
report_power >> $power_rpt

power_report_put ""
power_report_put "=== at the sweep's input activity, one code point per clock ==="

set total 0
set seen 0
set applied 0
foreach port [get_ports] {
    incr total
    set name [get_name $port]
    if {![regexp {^\\?code\[([0-9]+)\]$} $name -> bit]} {
        continue
    }
    incr seen
    set activity [expr {1.0 / double(1 << $bit)}]
    if {[catch {set_power_activity -input_ports $port -activity $activity -duty 0.5} msg]} {
        power_report_put [format "  code\[%d\]: failed: %s" $bit $msg]
        continue
    }
    power_report_put [format "  code\[%d\]: activity %.6g per clock, duty 0.5" $bit $activity]
    incr applied
}
if {$seen == 0} {
    power_report_put [format "error: none of the %d ports is a code bit, so nothing was annotated" $total]
    exit 1
}
if {$applied != $seen} {
    power_report_put [format "error: annotated %d of %d code bits, so the second report would rest on a partial annotation" $applied $seen]
    exit 1
}
power_report_put [format "  annotated %d of %d code bits, %d of %d ports were not code bits" $applied $seen [expr {$total - $seen}] $total]
report_power >> $power_rpt
