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
# Power for the Tagma demo with the workload's input activity (issue #70).
#
# OpenSTA computes switching power from per-pin activity. With nothing set, the
# inputs sit at the tool default (activity 0.1, duty 0.5), so only the clock and
# the logic it forces switch and the data path is effectively static. This
# script annotates the primary inputs with the activity the demo actually
# presents; OpenSTA propagates it inward, so the data path is counted too.
#
# The basis is the stimulus, not a saved trace. The demo sweeps all 65,536 code
# points, one per board clock of 83.33 ns. Bit b of a counter incremented once
# per clock changes value 65536 / 2^b times per sweep of 65,536 clocks, so its
# transition rate is f / 2^b and its duty is 0.5. Every annotation is printed,
# and the report is produced twice, at the tool default and at the sweep, so the
# comparison is part of the output.
#
# Inputs, all required except the SPEF:
#   POWER_LIBERTY  sky130hd liberty
#   POWER_NETLIST  the netlist the flow routed
#   POWER_SDC      its SDC
#   POWER_TOP      top module name (default tagma_demo_top)
#   POWER_SPEF     optional parasitic file; without it the switching power
#                  carries no wire capacitance
#
# Run inside the ORFS image, driven by hw/openroad/run.sh.

foreach varname {POWER_LIBERTY POWER_NETLIST POWER_SDC} {
    if {![info exists ::env($varname)]} {
        puts "error: $varname is not set"
        exit 1
    }
}

set liberty $::env(POWER_LIBERTY)
set netlist $::env(POWER_NETLIST)
set sdc     $::env(POWER_SDC)
set top     [expr {[info exists ::env(POWER_TOP)] ? $::env(POWER_TOP) : "tagma_demo_top"}]

foreach f [list $liberty $netlist $sdc] {
    if {![file exists $f]} {
        puts "error: $f does not exist"
        exit 1
    }
}

read_liberty $liberty
read_verilog $netlist
link_design $top
read_sdc $sdc
set_propagated_clock [all_clocks]

if {[info exists ::env(POWER_SPEF)] && [file exists $::env(POWER_SPEF)]} {
    read_spef $::env(POWER_SPEF)
    puts "parasitics: $::env(POWER_SPEF)"
} else {
    puts "parasitics: none, so the switching power carries no wire capacitance"
}

set period 83.33e-9
set f_clk  [expr {1.0 / $period}]
puts [format "clock: %.6g Hz, period %.2f ns" $f_clk [expr {$period * 1e9}]]

puts "\n=== at the tool default input activity (0.1, duty 0.5) ==="
set_power_activity -input -activity 0.1 -duty 0.5
report_power

puts "\n=== at the sweep's input activity, one code point per clock ==="

# Only the property route is documented, so the accessors are tried in order
# and inside catch: an accessor this build does not have must report rather
# than abort the run.
proc port_name {port} {
    foreach cmd {get_name get_full_name} {
        if {[llength [info commands $cmd]] == 0} {
            continue
        }
        if {![catch {$cmd $port} name] && $name ne ""} {
            return $name
        }
    }
    if {![catch {get_property $port name} name] && $name ne ""} {
        return $name
    }
    return ""
}

set total 0
set unreadable 0
set seen 0
set applied 0
foreach_in_collection port [get_ports] {
    incr total
    set name [port_name $port]
    if {$name eq ""} {
        incr unreadable
        continue
    }
    if {![regexp {^\\?code\[([0-9]+)\]$} $name -> bit]} {
        continue
    }
    incr seen
    set activity [expr {$f_clk / double(1 << $bit)}]
    if {[catch {set_power_activity -input_ports $port -activity $activity -duty 0.5} msg]} {
        puts [format "  code\[%d\]: failed: %s" $bit $msg]
        continue
    }
    puts [format "  code\[%d\]: activity %.6g duty 0.5" $bit $activity]
    incr applied
}
if {$seen == 0} {
    puts [format "error: none of the %d ports is a code bit (%d names could not be read), so nothing was annotated" $total $unreadable]
    exit 1
}
if {$applied != $seen} {
    puts [format "error: annotated %d of %d code bits, so the second report would rest on a partial annotation" $applied $seen]
    exit 1
}
puts [format "  annotated %d of %d code bits, %d of %d ports were not code bits" $applied $seen [expr {$total - $seen}] $total]
report_power
