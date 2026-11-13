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
// Self-checking streaming top.
//
// The pipeline and the combinational decoder run from the same code counter,
// the reference is delayed by the pipeline depth, and every clock compares the
// two triples. This carries the simulation evidence onto silicon: the streams
// agree at one code point per clock for as long as the board runs.
//
// The 25 MHz oscillator is doubled by pipe_pll, which is the only way to show
// the pipeline above the board rate. Sweep completion is counted rather than
// displayed directly, so the time from configuration to led_swept is itself a
// throughput reading: 2^28 clocks is 6.71 s at 40 MHz and 10.74 s at 25 MHz.
//
// Signals are named by function and constrained by ball. The vendor README
// gives the RGB balls in the order {A11, A12, B11} without naming a colour per
// ball, so the design does not rely on the colour mapping.

module tagma_pipe_check_top (
    input  wire clk,        // 25 MHz crystal on P6
    output wire led_clk,    // blinks while the fast clock runs
    output wire led_err,    // lights and stays lit on any mismatch
    output wire led_swept   // lights once the comparison count is reached
);
    wire clk_fast;

    pipe_pll pll (
        .clk_in(clk),
        .clk(clk_fast),
        .locked()
    );

    // One code point per clock over the valid range.
    reg [15:0] code = 16'hAC00;

    always @(posedge clk_fast)
        code <= (code >= 16'hD7A3) ? 16'hAC00 : code + 16'd1;

    wire [4:0] ri, rm, rf;

    tagma_decoder ref_dec (
        .code(code),
        .i(ri),
        .m(rm),
        .f(rf)
    );

    wire [4:0] pi, pm, pf;

    tagma_decoder_pipe dut (
        .clk(clk_fast),
        .code(code),
        .i(pi),
        .m(pm),
        .f(pf)
    );

    // The reference catches up with the pipeline's two-stage latency.
    reg [4:0] d1i, d1m, d1f;
    reg [4:0] r2i, r2m, r2f;

    always @(posedge clk_fast) begin
        d1i <= ri;
        d1m <= rm;
        d1f <= rf;
        r2i <= d1i;
        r2m <= d1m;
        r2f <= d1f;
    end

    reg err = 1'b0;

    always @(posedge clk_fast)
        if ({pi, pm, pf} !== {r2i, r2m, r2f})
            err <= 1'b1;

    // Saturating count, so the elapsed time to led_swept is a rate reading.
    // 28'hFFFFFFF is 2^28 - 1 clocks: 6.71 s at 40 MHz, 10.74 s at 25 MHz.
    reg [27:0] compared = 28'd0;

    always @(posedge clk_fast)
        if (compared != 28'hFFFFFFF)
            compared <= compared + 28'd1;

    reg [23:0] heart = 24'd0;

    always @(posedge clk_fast)
        heart <= heart + 24'd1;

    assign led_clk   = heart[23];
    assign led_err   = err;
    assign led_swept = (compared == 28'hFFFFFFF) && !err;
endmodule
