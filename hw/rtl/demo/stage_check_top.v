// SPDX-License-Identifier: CERN-OHL-P-2.0
// Copyright (C) 2026 Taeho Lee
//
// This source describes Open Hardware and is licensed under the CERN-OHL-P v2.
// You may redistribute and modify this documentation and make products using
// it under the terms of the CERN-OHL-P v2 (https://cern.ch/cern-ohl). This
// documentation is distributed WITHOUT ANY EXPRESS OR IMPLIED WARRANTY,
// INCLUDING OF MERCHANTABILITY, SATISFACTORY QUALITY AND FITNESS FOR A
// PARTICULAR PURPOSE. Please see the CERN-OHL-P v2 for applicable conditions.
//
// The stage pins, straight onto the RGB channels.
//
// The volume image turns the stage into a colour, and if a colour never appears
// there are two possible causes: the stage did not arrive, or the colour logic
// did not run. This top removes the second one. Each bit of the stage is one
// channel, so a host that raises bit 0 sees red, bit 1 green and bit 2 blue, and
// anything else is the input path.
//
// No clock is involved, so this also says whether the part is configured at all.

module stage_check_top (
    input  wire [2:0] stage,
    output wire led_red,
    output wire led_green,
    output wire led_blue
);
    assign led_red   = stage[0];
    assign led_green = stage[1];
    assign led_blue  = stage[2];
endmodule
