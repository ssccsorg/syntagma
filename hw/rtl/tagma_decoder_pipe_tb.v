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
    // The run ends after this many clocks: the sweep, plus the clocks the
    // pipeline and the reference delay line need to drain.
    localparam integer CLOCKS  = SWEEPS * CODES + LATENCY + 2;

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

    // Non-blocking so Verilator does not flag a blocking assignment in a
    // delay-controlled process (BLKSEQ, fatal under -Wall).
    always #5 clk <= ~clk;

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

    // The counter owns the report. Under Verilator 5.020 a process that resumed
    // from a delay read these counters as the values it set at time zero, when it
    // also set them itself, so a separate initial block that waited out the sweep
    // reported nothing checked. Blocking updates keep the check and the report in
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
        if (cycles == CLOCKS) begin
            if (errors == 0) begin
                $display("PASS: %0d outputs checked over %0d clocks, %0d full sweeps of the valid space, one code point per clock",
                         checked, cycles, SWEEPS);
                $finish;
            end else begin
                $display("FAIL: %0d mismatches over %0d checked outputs", errors, checked);
                $fatal;
            end
        end
    end
    /* verilator lint_on BLKSEQ */

    initial begin
        clk     = 1'b0;
        code    = 16'hAC00;
        cycles  = 0;
        checked = 0;
        errors  = 0;
    end

    // A clock that never toggles leaves the block above silent, and silence
    // exits zero, so this run ends with a failure of its own. The margin is
    // simulated time, so a slow host cannot trip it.
    initial begin
        #(CLOCKS * 20 + 10000);
        $display("FAIL: the sweep did not finish; the clock did not advance");
        $fatal;
    end
endmodule
