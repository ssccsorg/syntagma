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
// On-device verification of the streaming decoder, read through the debugger.
//
// The check top next to this file compares the pipeline against the
// combinational unit on the same die. That shows the two instances agree, and
// it cannot show that either agrees with the reference implementation, because
// both are the same source. This top closes that gap: it accumulates a
// checksum over one full sweep of the valid range and compares it against
// EXPECTED, a constant the flow derives from hw/rtl/golden_anchors.hex, which
// tagma_core writes. The verdict leaves the die on a pin, so a host reads it
// without a person looking at an LED.
//
// The three outputs land on ECP5 balls the on-board iCELink reaches as GPIOs,
// so the host reads them with icesprog and no external wiring is involved:
//
//   pin_match -> F5, iCELink PA14
//   pin_beat  -> H5, iCELink PA13
//   pin_sweep -> N4, iCELink PA0
//
// pin_beat and pin_sweep are two clocks divided by different amounts, and the
// host measures both. Their ratio is fixed by the divider, so a clock that is
// not what the PLL was configured for shows up as a mismatch between the two.
//
// EXPECTED must be supplied by the flow. The default of zero never matches,
// because no sweep of the valid range has a zero checksum, so an unset
// parameter reads as a failure rather than as a pass.

module tagma_pipe_verify_top #(
    parameter [31:0] EXPECTED = 32'h00000000
) (
    input  wire clk_in,
    output wire pin_match,
    output wire pin_beat,
    output wire pin_sweep
);
    wire clk;

    pipe_pll pll (
        .clk_in(clk_in),
        .clk(clk),
        .locked()
    );

    // One code point per clock, wrapping at the end of the valid range.
    reg [15:0] code = 16'd0;

    always @(posedge clk)
        code <= (code >= 16'hD7A3) ? 16'hAC00 : code + 16'd1;

    wire [4:0] pi, pm, pf;

    tagma_decoder_pipe dut (
        .clk(clk),
        .code(code),
        .i(pi),
        .m(pm),
        .f(pf)
    );

    // The pipeline reports a code three clocks after it was presented, so the
    // window logic runs on a copy of the code delayed by that much.
    reg [15:0] cd1, cd2, cd3;

    always @(posedge clk) begin
        cd1 <= code;
        cd2 <= cd1;
        cd3 <= cd2;
    end

    // pack = (i << 10) | (m << 5) | f, the same packing the anchors use.
    wire [14:0] pack = {pi, pm, pf};

    reg [31:0] acc = 32'd0;

    wire [31:0] acc_next = acc + {17'd0, pack};

    reg        matched = 1'b0;
    reg [24:0] heart = 25'd0;
    reg [13:0] sweeps = 14'd0;

    always @(posedge clk) begin
        heart <= heart + 25'd1;

        if (cd3 == 16'hD7A3) begin
            // acc_next includes the last code point, so this is the whole sweep.
            matched <= (acc_next == EXPECTED);
            acc     <= 32'd0;
            sweeps  <= sweeps + 14'd1;
        end else if (cd3 >= 16'hAC00) begin
            acc <= acc_next;
        end
    end

    assign pin_match = matched;
    assign pin_beat  = heart[24];
    assign pin_sweep = sweeps[10];
endmodule
