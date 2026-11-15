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
// The anchor comparison against a fixture, so the top has a gate that does not
// need a board.
//
// The file comes from `+rom=` and the expected verdict from `+expect=`, so one
// build checks a passing file and a file with two packs swapped, which is the
// case a checksum cannot see. The pass is two sweeps long: the first may still
// be filling the pipeline when it starts.
//
//   +rom=out/anchor_fixture_pass.hex +expect=1
//   +rom=out/anchor_fixture_swap.hex +expect=0

module tagma_anchor_verify_tb #(
    parameter integer COUNT = 8
) ();
    reg clk = 1'b0;
    reg [1:0] stage = 2'd3;

    wire pin_match;
    wire led_red, led_green, led_blue;

    tagma_anchor_verify_top #(.COUNT(COUNT)) dut (
        .clk_in(clk),
        .stage(stage),
        .pin_match(pin_match),
        .led_red(led_red),
        .led_green(led_green),
        .led_blue(led_blue)
    );

    always #5 clk = ~clk;

    integer want = 1;
    integer guard;

    initial begin
        if (!$value$plusargs("expect=%d", want))
            want = 1;

        guard = 4 * (COUNT + 16);
        repeat (guard) @(posedge clk);

        if (pin_match !== (want != 0)) begin
            $display("FAIL: pin_match=%0d, expected %0d after %0d clocks over %0d lines",
                     pin_match, want, guard, COUNT);
            $fatal(1);
        end

        // Stage 3 is steady and carries the verdict, so the colour is the pin.
        // A run that says the right number in the wrong colour is still a fault.
        if (led_green !== pin_match || led_red !== !pin_match || led_blue !== 1'b0) begin
            $display("FAIL: stage 3 lit red=%0d green=%0d blue=%0d for pin_match=%0d",
                     led_red, led_green, led_blue, pin_match);
            $fatal(1);
        end

        $display("PASS: pin_match=%0d over %0d lines in %0d clocks",
                 pin_match, COUNT, guard);
        $finish;
    end
endmodule
