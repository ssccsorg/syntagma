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
// Streaming top for the pipelined decoder.
//
// The counter is the source register, so every timing path runs from a
// register to a register through the decoder. That is what a place and route
// tool needs in order to report a clock for the streaming case: the demo top
// takes its code from pins, so its paths are cross-domain and no Fmax is
// reported for it.

module tagma_pipe_top (
    input  wire        clk,
    output wire [4:0]  i,
    output wire [4:0]  m,
    output wire [4:0]  f
);
    reg [15:0] counter = 16'd0;

    always @(posedge clk)
        counter <= counter + 16'd1;

    tagma_decoder_pipe dut (
        .clk(clk),
        .code(counter),
        .i(i),
        .m(m),
        .f(f)
    );
endmodule
