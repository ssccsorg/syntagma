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
// Tagma 3-axis decoder, three-stage pipeline.
//
// Same arithmetic and the same result as tagma_decoder.v, split so that no
// stage carries more than one large constant multiply:
//
//   stage 1: offset -> j = (q4 * 9363) >> 16
//   stage 2: j -> i = (j * 781) >> 14
//   stage 3: m = j - 21 * i, f = offset - 28 * j
//
// The two large constants have no shift-and-add form, so they keep a
// multiplier each. The two small ones do: 21 = 16 + 4 + 1 and 28 = 32 - 4, so
// stage 3 is adders rather than multipliers, which is both shorter and two
// fewer MULT18X18D than the two-stage split.
//
// f is written as offset - 28 * j rather than 4 * (q4 - 7 * j) + r2. Both are
// the same value modulo 2^14, so the low five bits agree for every 16-bit
// input, valid or not.
//
// A code presented before a clock edge appears at the outputs three edges
// later, so the latency is three stages and the throughput is one code point
// per clock. The combinational unit stays the reference; this one is checked
// against it at every cycle, which carries the golden-anchor evidence over
// unchanged.

module tagma_decoder_pipe (
    input  wire        clk,
    input  wire [15:0] code,
    output reg  [4:0]  i,
    output reg  [4:0]  m,
    output reg  [4:0]  f
);
    // Stage 1: the first divide, which leaves j = offset / 28 in [0, 398].
    /* verilator lint_off UNUSEDSIGNAL */
    wire [15:0] sub = code - 16'hAC00;
    wire [13:0] offset = sub[13:0];
    wire [11:0] q4 = offset[13:2];
    wire [24:0] jp = {13'd0, q4} * 25'd9363;
    /* verilator lint_on UNUSEDSIGNAL */

    reg [8:0]  j_q;
    reg [13:0] off_q;

    always @(posedge clk) begin
        j_q   <= jp[24:16];
        off_q <= offset;
    end

    // Stage 2: the second divide, which leaves i = j / 21 in [0, 18].
    /* verilator lint_off UNUSEDSIGNAL */
    wire [18:0] ip = {10'd0, j_q} * 19'd781;
    /* verilator lint_on UNUSEDSIGNAL */
    wire [4:0]  ii = ip[18:14];

    reg [4:0]  i_q;
    reg [8:0]  j_q2;
    reg [13:0] off_q2;

    always @(posedge clk) begin
        i_q    <= ii;
        j_q2   <= j_q;
        off_q2 <= off_q;
    end

    // Stage 3: the two remainders, with the small constants as shifts.
    /* verilator lint_off UNUSEDSIGNAL */
    wire [10:0] i5  = {6'd0, i_q} + {4'd0, i_q, 2'b00};            // 5 * i
    wire [12:0] i21 = {i5, 2'b00} + {8'd0, i_q};                  // 21 * i
    wire [12:0] mm  = {4'd0, j_q2} - i21;
    wire [13:0] j32 = {j_q2, 5'b00000};
    wire [13:0] j4  = {3'd0, j_q2, 2'b00};
    wire [13:0] ff  = off_q2 - (j32 - j4);                        // offset - 28 * j
    /* verilator lint_on UNUSEDSIGNAL */

    always @(posedge clk) begin
        i <= i_q;
        m <= mm[4:0];
        f <= ff[4:0];
    end
endmodule
