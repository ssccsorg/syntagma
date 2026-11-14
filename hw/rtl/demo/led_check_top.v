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
// The RGB channels, one combination at a time.
//
// A check that only ever shows one colour cannot tell a working channel from a
// wrong ball, a wrong level, or a channel that never lights. This top steps
// through every combination of the three, so a person looking at the board can
// say which of those it is without a host in the loop.
//
// The step is about 1.3 seconds at the board's 25 MHz clock, which is slow
// enough to name the colour aloud.

module led_check_top (
    input  wire clk_in,
    output wire led_red,
    output wire led_green,
    output wire led_blue
);
    reg [27:0] count = 28'd0;

    always @(posedge clk_in)
        count <= count + 28'd1;

    wire [2:0] phase = count[27:25];

    // 0 red, 1 green, 2 blue, 3 red+green, 4 green+blue, 5 red+blue, 6 all, 7 off
    reg red, green, blue;

    always @* begin
        case (phase)
            3'd0:    {red, green, blue} = 3'b100;
            3'd1:    {red, green, blue} = 3'b010;
            3'd2:    {red, green, blue} = 3'b001;
            3'd3:    {red, green, blue} = 3'b110;
            3'd4:    {red, green, blue} = 3'b011;
            3'd5:    {red, green, blue} = 3'b101;
            3'd6:    {red, green, blue} = 3'b111;
            default: {red, green, blue} = 3'b000;
        endcase
    end

    assign led_red   = red;
    assign led_green = green;
    assign led_blue  = blue;
endmodule
