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
// Tagma segment store: the coordinate space as a flat 16-bit memory.
//
// 11,172 slots of 16 bits, addressed by the 14-bit coordinate index
// (code - U+AC00), the behavioral model of the chton SRAM macro configured in
// hw/openram/chton_sram.py. Single port: writes are clocked and reads are
// registered, so a slot is available one cycle after its address is presented.
// The 14-bit address space covers 16,384 slots; 11,172..16,383 are reserved,
// where writes are ignored and reads return zero.

module tagma_segment_store (
    input  wire        clk,
    input  wire        we,
    input  wire [13:0] addr,
    input  wire [15:0] wdata,
    output reg  [15:0] rdata
);
    reg [15:0] mem [0:11171];

    wire        valid = (addr < 14'd11172);
    wire [13:0] idx   = valid ? addr : 14'd0;

    always @(posedge clk) begin
        if (we && valid) mem[idx] <= wdata;
        rdata <= valid ? mem[idx] : 16'h0000;
    end
endmodule
