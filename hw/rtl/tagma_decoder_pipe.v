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
// Tagma 3-axis decoder, two-stage pipeline.
//
// Same arithmetic and the same constants as tagma_decoder.v, split so that
// each stage carries one multiplier:
//
//   stage 1: offset -> q4, r2 -> j = (q4 * 9363) >> 16
//   stage 2: j -> i = (j * 781) >> 14, m = j % 21, f = 4 * (q4 % 7) + r2
//
// A code presented before a clock edge appears at the outputs two edges later,
// so the latency is two stages and the throughput is one code point per clock.
// The combinational unit stays the reference; this one is checked against it
// at every cycle, which carries the golden-anchor evidence over unchanged.

module tagma_decoder_pipe (
    input  wire        clk,
    input  wire [15:0] code,
    output reg  [4:0]  i,
    output reg  [4:0]  m,
    output reg  [4:0]  f
);
    // Stage 1 holds everything the second stage needs from this code point.
    reg [11:0] q4_q;
    reg [1:0]  r2_q;
    reg [8:0]  j_q;

    /* verilator lint_off UNUSEDSIGNAL */
    wire [15:0] sub = code - 16'hAC00;
    wire [13:0] offset = sub[13:0];

    // Split offset = 4*q4 + r2; 28 = 4 * 7 keeps the multiplier small.
    wire [11:0] q4 = offset[13:2];
    wire [1:0]  r2 = offset[1:0];

    // j = offset / 28 in [0, 398]: (q4 * 9363) >> 16.
    wire [24:0] jp = {13'd0, q4} * 25'd9363;
    /* verilator lint_on UNUSEDSIGNAL */

    always @(posedge clk) begin
        q4_q <= q4;
        r2_q <= r2;
        j_q  <= jp[24:16];
    end

    // Stage 2: the second multiplier and the remainders.
    /* verilator lint_off UNUSEDSIGNAL */
    wire [18:0] ip = {10'd0, j_q} * 19'd781;
    wire [4:0]  ii = ip[18:14];
    wire [9:0]  mm = {1'd0, j_q} - {5'd0, ii} * 10'd21;
    wire [11:0] rem7 = q4_q - {3'd0, j_q} * 12'd7;
    wire [13:0] ff = {rem7, 2'd0} + {12'd0, r2_q};
    /* verilator lint_on UNUSEDSIGNAL */

    always @(posedge clk) begin
        i <= ii;
        m <= mm[4:0];
        f <= ff[4:0];
    end
endmodule
