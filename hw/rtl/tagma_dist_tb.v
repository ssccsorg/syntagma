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
//                 syllable, both operand orders, plus a sweep where both
//                 operands vary (make sim-dist)
//   GOLDEN_CHECK: the same over the exported decomposition, checked against the
//                 golden anchors (make sim-dist-golden)

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
`else
    integer j;            // the second operand's offset in the varying sweep
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

        // Both operands vary: b is a stride-7 rotation of a, so both decode
        // paths move and both operand orders occur. The index is inlined, since
        // golden's line index is the offset the line holds.
        for (k = 0; k < 11172; k = k + 1) begin
            a  = 16'hAC00 + {2'd0, golden[k][28:15]};
            b  = 16'hAC00 + {2'd0, golden[(k * 7 + 3) % 11172][28:15]};
            ai = {27'd0, golden[k][14:10]};
            am = {27'd0, golden[k][9:5]};
            af = {27'd0, golden[k][4:0]};
            bi = {27'd0, golden[(k * 7 + 3) % 11172][14:10]};
            bm = {27'd0, golden[(k * 7 + 3) % 11172][9:5]};
            bf = {27'd0, golden[(k * 7 + 3) % 11172][4:0]};
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

        // Both operands vary: b is a stride-7 rotation of a, so both decode
        // paths move and both operand orders occur.
        for (k = 0; k < 11172; k = k + 1) begin
            j  = (k * 7 + 3) % 11172;
            a  = 16'hAC00 + k[15:0];
            b  = 16'hAC00 + j[15:0];
            ai = k / 588;
            am = (k % 588) / 28;
            af = k % 28;
            bi = j / 588;
            bm = (j % 588) / 28;
            bf = j % 28;
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
            $display("PASS: 3 x 11,172 golden distance checks over varying operands and both orders");
`else
            $display("PASS: 3 x 11,172 distances over varying operands and both orders");
`endif
        end else
            $display("FAIL: %0d mismatches", errors);

        $finish;
    end
endmodule
