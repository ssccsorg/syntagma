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
// Tagma 3-axis decoder, four-stage pipeline.
//
// Same arithmetic and the same result as tagma_decoder.v, split so that no
// stage carries more than one large constant multiply and neither multiply
// shares its stage with the offset subtraction:
//
//   stage 0: offset = code - 0xAC00
//   stage 1: j = (q4 * 9363) >> 16
//   stage 2: i = (j * 781) >> 14
//   stage 3: m = j - 21 * i, f = offset - 28 * j
//
// The two large constants have no shift-and-add form, so they keep a
// multiplier each. The two small ones do: 21 = 16 + 4 + 1 and 28 = 32 - 4, so
// stage 3 is adders rather than multipliers.
//
// f is written as offset - 28 * j rather than 4 * (q4 - 7 * j) + r2. Both are
// the same value modulo 2^14, so the low five bits agree for every 16-bit
// input, valid or not.
//
// The offset subtraction has a stage of its own because at three stages it was
// 1.2 ns of the 7.8 ns critical path into the first multiplier, and a register
// costs less than that.
//
// A code presented before a clock edge appears at the outputs four edges later,
// so the latency is four stages and the throughput is one code point per clock.
// The combinational unit stays the reference; this one is checked against it at
// every cycle, which carries the golden-anchor evidence over unchanged.

module tagma_decoder_pipe (
    input  wire        clk,
    input  wire [15:0] code,
    output reg  [4:0]  i,
    output reg  [4:0]  m,
    output reg  [4:0]  f
);
    // Stage 0: the offset, which is what every later stage works from.
    /* verilator lint_off UNUSEDSIGNAL */
    wire [15:0] sub = code - 16'hAC00;
    wire [13:0] offset = sub[13:0];
    /* verilator lint_on UNUSEDSIGNAL */

    reg [13:0] off_d0;

    always @(posedge clk)
        off_d0 <= offset;

    // Stage 1: the first divide, which leaves j = offset / 28 in [0, 398].
    /* verilator lint_off UNUSEDSIGNAL */
    wire [11:0] q4 = off_d0[13:2];
    wire [24:0] jp = {13'd0, q4} * 25'd9363;
    /* verilator lint_on UNUSEDSIGNAL */

    reg [8:0]  j_d1;
    reg [13:0] off_d1;

    always @(posedge clk) begin
        j_d1   <= jp[24:16];
        off_d1 <= off_d0;
    end

    // Stage 2: the second divide, which leaves i = j / 21 in [0, 18].
    /* verilator lint_off UNUSEDSIGNAL */
    wire [18:0] ip = {10'd0, j_d1} * 19'd781;
    /* verilator lint_on UNUSEDSIGNAL */
    wire [4:0]  ii = ip[18:14];

    reg [4:0]  i_d2;
    reg [8:0]  j_d2;
    reg [13:0] off_d2;

    always @(posedge clk) begin
        i_d2   <= ii;
        j_d2   <= j_d1;
        off_d2 <= off_d1;
    end

    // Stage 3: the two remainders, with the small constants as shifts.
    /* verilator lint_off UNUSEDSIGNAL */
    wire [10:0] i5  = {6'd0, i_d2} + {4'd0, i_d2, 2'b00};          // 5 * i
    wire [12:0] i21 = {i5, 2'b00} + {8'd0, i_d2};                 // 21 * i
    wire [12:0] mm  = {4'd0, j_d2} - i21;
    wire [13:0] j32 = {j_d2, 5'b00000};
    wire [13:0] j4  = {3'd0, j_d2, 2'b00};
    wire [13:0] ff  = off_d2 - (j32 - j4);                        // offset - 28 * j
    /* verilator lint_on UNUSEDSIGNAL */

    always @(posedge clk) begin
        i <= i_d2;
        m <= mm[4:0];
        f <= ff[4:0];
    end
endmodule
