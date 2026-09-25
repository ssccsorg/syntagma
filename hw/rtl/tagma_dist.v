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
// Tagma distance: field-wise absolute difference between two coordinates.
//
// Both operands are 16-bit code points, the domain the decoder takes. Each is
// decoded by the same tagma_decoder the standalone decoder uses, so both share
// one decode definition. The three 5-bit distances are returned separately;
// for a valid pair they match Coord::hamming_distance. As with the decoder, an
// out-of-range operand has no defined decode and no validity flag is raised:
// the caller supplies valid code points.

module tagma_dist (
    input  wire [15:0] a,
    input  wire [15:0] b,
    output wire [4:0]  di,
    output wire [4:0]  dm,
    output wire [4:0]  df
);
    wire [4:0] ai, am, af;
    wire [4:0] bi, bm, bf;

    tagma_decoder da (.code(a), .i(ai), .m(am), .f(af));
    tagma_decoder db (.code(b), .i(bi), .m(bm), .f(bf));

    assign di = (ai > bi) ? (ai - bi) : (bi - ai);
    assign dm = (am > bm) ? (am - bm) : (bm - am);
    assign df = (af > bf) ? (af - bf) : (bf - af);
endmodule
