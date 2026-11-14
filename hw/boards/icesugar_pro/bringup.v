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
// Bring-up design for the iCESugar-Pro v1.3 (ECP5 LFE5U-25F-6BG256C).
//
// The board alone carries only the RGB LED, so the three channels are driven
// from separate counter bits. That shows the design is running without
// external wiring and without assuming which channel is which.
// Pin map: hw/boards/icesugar_pro/bringup.lpf

module bringup (
    input  wire clk,      // 25 MHz crystal on P6
    output wire led_r,
    output wire led_g,
    output wire led_b
);
    reg [26:0] counter = 27'd0;

    always @(posedge clk)
        counter <= counter + 27'd1;

    assign led_r = counter[24];
    assign led_g = counter[25];
    assign led_b = counter[26];
endmodule
