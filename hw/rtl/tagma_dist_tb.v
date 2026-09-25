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
// Exhaustive testbench for tagma_dist.
//
//   default:      every valid code point against the first and the last
//                 syllable, which exercises both operand orders (make sim-dist)
//   GOLDEN_CHECK: the same over the exported decomposition, so the distance is
//                 checked against the golden anchors (make sim-dist-golden)

`timescale 1ns/1ps

module tagma_dist_tb;
    reg  [15:0] a;
    reg  [15:0] b;
    wire [4:0]  di;
    wire [4:0]  dm;
    wire [4:0]  df;

    integer errors;
    integer k;
    integer ai, am, af;   // axes of a
    integer bi, bm, bf;   // axes of b
    integer ddi, ddm, ddf;

`ifdef GOLDEN_CHECK
    reg [28:0] golden [0:11171];
`endif

    tagma_dist dut (
        .a(a),
        .b(b),
        .di(di),
        .dm(dm),
        .df(df)
    );

    // The reference is Coord::hamming_distance: the field-wise absolute
    // difference of the two axis triples.
    task check;
        begin
            #1;
            ddi = (ai > bi) ? (ai - bi) : (bi - ai);
            ddm = (am > bm) ? (am - bm) : (bm - am);
            ddf = (af > bf) ? (af - bf) : (bf - af);
            if (ddi !== {27'd0, di} || ddm !== {27'd0, dm} || ddf !== {27'd0, df}) begin
                $display("MISMATCH a=0x%04h b=0x%04h got %0d/%0d/%0d expected %0d/%0d/%0d",
                         a, b, di, dm, df, ddi, ddm, ddf);
                errors = errors + 1;
            end
        end
    endtask

    initial begin
        errors = 0;

`ifdef GOLDEN_CHECK
        $readmemh("golden_anchors.hex", golden);

        // Distance to U+AC00 (axes 0,0,0): the difference is the axis itself.
        bi = 0; bm = 0; bf = 0;
        b = 16'hAC00;
        for (k = 0; k < 11172; k = k + 1) begin
            a  = 16'hAC00 + {2'd0, golden[k][28:15]};
            ai = {27'd0, golden[k][14:10]};
            am = {27'd0, golden[k][9:5]};
            af = {27'd0, golden[k][4:0]};
            check;
        end

        // Distance to U+D7A3 (axes 18,20,27): reverses the operand order.
        bi = 18; bm = 20; bf = 27;
        b = 16'hD7A3;
        for (k = 0; k < 11172; k = k + 1) begin
            a  = 16'hAC00 + {2'd0, golden[k][28:15]};
            ai = {27'd0, golden[k][14:10]};
            am = {27'd0, golden[k][9:5]};
            af = {27'd0, golden[k][4:0]};
            check;
        end
`else
        // Distance to U+AC00 (axes 0,0,0): the difference is the axis itself.
        bi = 0; bm = 0; bf = 0;
        b = 16'hAC00;
        for (k = 0; k < 11172; k = k + 1) begin
            a  = 16'hAC00 + k[15:0];
            ai = k / 588;
            am = (k % 588) / 28;
            af = k % 28;
            check;
        end

        // Distance to U+D7A3 (axes 18,20,27): reverses the operand order.
        bi = 18; bm = 20; bf = 27;
        b = 16'hD7A3;
        for (k = 0; k < 11172; k = k + 1) begin
            a  = 16'hAC00 + k[15:0];
            ai = k / 588;
            am = (k % 588) / 28;
            af = k % 28;
            check;
        end

        // A coordinate is at distance zero from itself.
        ai = 0; am = 0; af = 0;
        bi = 0; bm = 0; bf = 0;
        a = 16'hAC00; b = 16'hAC00;
        check;
`endif

        if (errors == 0) begin
`ifdef GOLDEN_CHECK
            $display("PASS: 2 x 11,172 golden distance anchors verified");
`else
            $display("PASS: 2 x 11,172 distances verified over both operand orders");
`endif
        end else
            $display("FAIL: %0d mismatches", errors);

        $finish;
    end
endmodule
