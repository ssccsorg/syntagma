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
// Testbench for tagma_demo_top.
//
//   default:      every one of the 65,536 code points. The three axes are
//                 checked against the formula on the 11,172 valid syllables,
//                 and the validity LED is checked on all of them
//                 (make sim-demo)
//   ASC_CHECK:    additionally instantiates the placed-and-routed bitstream,
//                 recovered to Verilog by icebox_vlog, and compares every
//                 output against the RTL on all 65,536 code points
//                 (make sim-pnr)
//
// The axes are defined on the valid domain only. The decoder subtracts
// U+AC00 before dividing, so an input below the block underflows and an input
// above it leaves the index out of range; `valid` in tagma_demo_top is what
// separates the two cases, and it is the quantity the LED carries. The
// comparison mode checks the axes outside the valid domain as well, because
// there the question is only whether the bitstream agrees with the RTL.
//
// The expected values are computed in 32-bit integers, the way the compose
// and distance testbenches do, so no width warning is raised under -Wall.

`timescale 1ns/1ps

module tagma_demo_top_tb;
    reg         clk;
    reg  [15:0] code;

    wire [4:0]  i;
    wire [4:0]  m;
    wire [4:0]  f;
    wire        led_valid;

    integer errors;
    integer k, off;
    integer ei, em, ef;
    reg     ev;

`ifdef ASC_CHECK
    wire [4:0]  i_asc;
    wire [4:0]  m_asc;
    wire [4:0]  f_asc;
    wire        led_asc;
`endif

    tagma_demo_top dut (
        .clk(clk),
        .code(code),
        .i(i),
        .m(m),
        .f(f),
        .led_valid(led_valid)
    );

`ifdef ASC_CHECK
    // The .asc recovered by icebox_vlog in synth/yosys/run_pnr.sh. Its port
    // names come back from the constraints file as escaped identifiers, so the
    // same stimulus drives both instances and the outputs are compared after
    // every clock edge.
    tagma_demo_top_postpnr asc (
        .clk(clk),
        .\code (code),
        .\i (i_asc),
        .\m (m_asc),
        .\f (f_asc),
        .led_valid(led_asc)
    );
`endif

    // Non-blocking so Verilator does not flag a blocking assignment in a
    // delay-controlled process (BLKSEQ, fatal under -Wall).
    always #1 clk <= ~clk;

    initial begin
        clk    = 1'b0;
        code   = 16'd0;
        errors = 0;

        for (k = 0; k < 65536; k = k + 1) begin
            @(negedge clk);
            code = k[15:0];
            @(posedge clk);
            #1;

            if (k >= 44032 && k <= 55203) begin
                off = k - 44032;
                ei  = off / 588;
                em  = (off % 588) / 28;
                ef  = off % 28;
                ev  = 1'b1;
            end else begin
                ei = 0;
                em = 0;
                ef = 0;
                ev = 1'b0;
            end

            // led_valid is negative logic, so it is the inversion of `valid`.
            if (led_valid !== ~ev) begin
                $display("LED MISMATCH code=0x%04h got=%b expected=%b", code, led_valid, ~ev);
                errors = errors + 1;
            end

            if (ev && ({27'd0, i} !== ei || {27'd0, m} !== em || {27'd0, f} !== ef)) begin
                $display("AXIS MISMATCH code=0x%04h got=%0d/%0d/%0d expected=%0d/%0d/%0d",
                         code, i, m, f, ei, em, ef);
                errors = errors + 1;
            end

`ifdef ASC_CHECK
            if (i !== i_asc || m !== m_asc || f !== f_asc || led_valid !== led_asc) begin
                $display("BITSTREAM MISMATCH code=0x%04h rtl=%0d/%0d/%0d/%b bitstream=%0d/%0d/%0d/%b",
                         code, i, m, f, led_valid, i_asc, m_asc, f_asc, led_asc);
                errors = errors + 1;
            end
`endif
        end

        if (errors == 0) begin
`ifdef ASC_CHECK
            $display("PASS: the demo top and the placed-and-routed bitstream agree over all 65,536 code points");
`else
            $display("PASS: 11,172 valid axes and 65,536 validity LED values verified");
`endif
        end else
            $display("FAIL: %0d mismatches", errors);

        $finish;
    end
endmodule
