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
// Tagma compose: the decoder read backwards, from the three axes to the index.
//
//   index = 588*i + 28*m + f,  i in [0,18], m in [0,20], f in [0,27]
//
// Factored as 28*(21*i + m) + f, so the multipliers are 21*i and 28*p, the
// same shape the decoder's constant division has. An axis combination outside
// the bounds reports valid = 0 and index = 0.
//
// Each product declares its full width and is sliced, the style the decoder's
// multiply networks use. The high bits of the products are zero over the valid
// domain, so UNUSEDSIGNAL is suppressed around them, as in the decoder.

module tagma_compose (
    input  wire [4:0]  i,
    input  wire [4:0]  m,
    input  wire [4:0]  f,
    output wire        valid,
    output wire [13:0] index
);
    // p = 21*i + m, at most 398 for a valid combination.
    // index = 28*p + f, at most 11171 for a valid combination.
    /* verilator lint_off UNUSEDSIGNAL */
    wire [19:0] i21 = {5'd0, i} * 10'd21;
    wire [9:0]  p   = i21[9:0] + {5'd0, m};
    wire [27:0] p28 = {4'd0, p} * 14'd28;
    wire [13:0] q   = p28[13:0] + {9'd0, f};
    /* verilator lint_on UNUSEDSIGNAL */

    assign valid = (i < 5'd19) && (m < 5'd21) && (f < 5'd28);
    assign index = valid ? q : 14'd0;
endmodule
