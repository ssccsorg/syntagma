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
// Exhaustive testbench for tagma_compose.
//
//   default:      all 32^3 axis combinations, checked against the formula
//                 and the validity bounds (make sim-compose)
//   GOLDEN_CHECK: the 11,172 (i, m, f) triples exported from tagma_core's
//                 Coord::to_axes, each checked to recover its offset
//                 (make sim-compose-golden)

`timescale 1ns/1ps

module tagma_compose_tb;
    reg  [4:0]  i;
    reg  [4:0]  m;
    reg  [4:0]  f;
    wire        valid;
    wire [13:0] index;

    integer errors;

`ifdef GOLDEN_CHECK
    integer    k;
    reg [28:0] golden [0:11171];
`else
    integer    ii, mm, ff;
    integer    exp_index;
    reg        exp_valid;
`endif

    tagma_compose dut (
        .i(i),
        .m(m),
        .f(f),
        .valid(valid),
        .index(index)
    );

    initial begin
        errors = 0;

`ifdef GOLDEN_CHECK
        $readmemh("golden_anchors.hex", golden);

        for (k = 0; k < 11172; k = k + 1) begin
            i = golden[k][14:10];
            m = golden[k][9:5];
            f = golden[k][4:0];
            #1;
            if (!valid || index !== golden[k][28:15]) begin
                $display("GOLDEN MISMATCH line=%0d i=%0d m=%0d f=%0d got valid=%b index=%0d expected index=%0d",
                         k, i, m, f, valid, index, golden[k][28:15]);
                errors = errors + 1;
            end
        end
`else
        for (ii = 0; ii < 32; ii = ii + 1)
            for (mm = 0; mm < 32; mm = mm + 1)
                for (ff = 0; ff < 32; ff = ff + 1) begin
                    i = ii[4:0];
                    m = mm[4:0];
                    f = ff[4:0];
                    exp_valid = (ii < 19) && (mm < 21) && (ff < 28);
                    exp_index = exp_valid ? (588*ii + 28*mm + ff) : 0;
                    #1;
                    if (valid !== exp_valid || {18'd0, index} !== exp_index) begin
                        $display("MISMATCH i=%0d m=%0d f=%0d got valid=%b index=%0d expected valid=%b index=%0d",
                                 i, m, f, valid, index, exp_valid, exp_index[13:0]);
                        errors = errors + 1;
                    end
                end
`endif

        if (errors == 0) begin
`ifdef GOLDEN_CHECK
            $display("PASS: all 11,172 golden compose anchors verified");
`else
            $display("PASS: all 32,768 axis combinations verified");
`endif
        end else
            $display("FAIL: %0d mismatches", errors);

        $finish;
    end
endmodule
