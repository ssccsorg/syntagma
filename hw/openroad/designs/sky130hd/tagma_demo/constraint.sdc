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
# Tagma demo top, SkyWater 130nm (sky130hd).
# 12 MHz board-equivalent clock: CLOCK_PERIOD = 83.33 ns.
create_clock -period 83.33 -name clk [get_ports clk]
set_input_delay -clock clk 1.0 [all_inputs -no_clocks]
set_output_delay -clock clk 1.0 [all_outputs]
