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
// Streaming testbench for tagma_decoder_pipe.
//
// One code point per clock, forever, over the valid range. The reference is the
// combinational decoder, which the other channels already pin to the golden
// anchors over all 11,172 points, so an exact match at every cycle after the
// fill carries that evidence over to the pipeline and proves the throughput.
//
// The source register, the reference delay line, and the pipeline all advance
// on the same edge, so the pipe output after edge E belongs to the code that
// was presented at edge E-4. That is the four-stage latency.

`timescale 1ns/1ps

module tagma_decoder_pipe_tb;
    localparam integer LATENCY = 4;
    localparam integer CODES   = 11172;
    localparam integer SWEEPS   = 4;

    reg clk;
    reg [15:0] code;

    wire [4:0] pi, pm, pf;

    tagma_decoder_pipe dut (
        .clk(clk),
        .code(code),
        .i(pi),
        .m(pm),
        .f(pf)
    );

    wire [4:0] ci, cm, cf;

    tagma_decoder ref_dec (
        .code(code),
        .i(ci),
        .m(cm),
        .f(cf)
    );

    always #5 clk = ~clk;

    // One code point per clock, wrapping at the end of the valid range.
    always @(posedge clk)
        code <= (code >= 16'hD7A3) ? 16'hAC00 : code + 16'd1;

    // Reference outputs delayed by the pipeline latency.
    reg [4:0] ri1, rm1, rf1;
    reg [4:0] ri2, rm2, rf2;
    reg [4:0] ri3, rm3, rf3;
    reg [4:0] ri4, rm4, rf4;

    always @(posedge clk) begin
        ri1 <= ci;
        rm1 <= cm;
        rf1 <= cf;
        ri2 <= ri1;
        rm2 <= rm1;
        rf2 <= rf1;
        ri3 <= ri2;
        rm3 <= rm2;
        rf3 <= rf2;
        ri4 <= ri3;
        rm4 <= rm3;
        rf4 <= rf3;
    end

    integer cycles;
    integer checked;
    integer errors;

    // A testbench counter: blocking updates keep the check and the report in
    // step, and this block is never synthesized.
    /* verilator lint_off BLKSEQ */
    always @(negedge clk) begin
        cycles = cycles + 1;
        if (cycles > LATENCY) begin
            if ((pi !== ri4) || (pm !== rm4) || (pf !== rf4)) begin
                if (errors < 5)
                    $display("MISMATCH at cycle %0d: pipe %0d/%0d/%0d, reference %0d/%0d/%0d",
                             cycles, pi, pm, pf, ri4, rm4, rf4);
                errors = errors + 1;
            end
            checked = checked + 1;
        end
    end
    /* verilator lint_on BLKSEQ */

    initial begin
        clk    = 1'b0;
        code   = 16'hAC00;
        cycles = 0;
        checked = 0;
        errors  = 0;
        repeat (SWEEPS * CODES + LATENCY + 2) @(negedge clk);
        if (errors == 0)
            $display("PASS: %0d outputs checked over %0d clocks, %0d full sweeps of the valid space, one code point per clock",
                     checked, cycles, SWEEPS);
        else
            $display("FAIL: %0d mismatches over %0d checked outputs", errors, checked);
        $finish;
    end
endmodule
