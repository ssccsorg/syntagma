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
// Tagma decoder FPGA demo top module.
//
// Targets the Upduino 3.1 (iCE40UP5K-SG48). The 16-bit code point comes
// in on GPIO switches, the three axes drive LED groups, and the onboard
// green LED lights when the code point is a valid Hangul syllable
// (0xAC00..0xD7A3). The decode is combinational; the outputs are
// registered on the board clock so the one-cycle latency and the timing
// closure are measurable with a real clock.
//
// The onboard LED is negative logic (write 0 to light), so led_valid
// carries the inverted validity signal. Pin mapping:
// hw/boards/upduino31/tagma_demo.pcf

module tagma_demo_top #(
    // A board whose LED lights on a high level sets this to 0. The Upduino's
    // onboard LED is negative logic; the iCESugar-Pro RGB LED is not.
    parameter LED_ACTIVE_LOW = 1
) (
    input  wire        clk,
    input  wire [15:0] code,
    output reg  [4:0]  i,
    output reg  [4:0]  m,
    output reg  [4:0]  f,
    output wire        led_valid
);
    wire [4:0] i_c;
    wire [4:0] m_c;
    wire [4:0] f_c;
    reg  valid;  // internal validity flag, drives the negative-logic LED

    tagma_decoder dec (
        .code(code),
        .i(i_c),
        .m(m_c),
        .f(f_c)
    );

    always @(posedge clk) begin
        i     <= i_c;
        m     <= m_c;
        f     <= f_c;
        valid <= (code >= 16'hAC00) && (code <= 16'hD7A3);
    end

    // Light the validity LED, inverting first on a negative-logic board.
    assign led_valid = LED_ACTIVE_LOW ? ~valid : valid;
endmodule
