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
// Exhaustive testbench for tagma_segment_store.
//
// Writes every slot with the code point that maps to it (U+AC00 + index) and
// reads it back, over all 11,172 valid slots. Then checks that the reserved
// addresses 11,172..16,383 ignore writes and read as zero. The simulation
// command lives in hw/Makefile (make sim-store).

`timescale 1ns/1ps

module tagma_segment_store_tb;
    reg         clk;
    reg         we;
    reg  [13:0] addr;
    reg  [15:0] wdata;
    wire [15:0] rdata;

    integer errors;
    integer k;

    tagma_segment_store dut (
        .clk(clk),
        .we(we),
        .addr(addr),
        .wdata(wdata),
        .rdata(rdata)
    );

    always #1 clk = ~clk;

    task check_slot;
        input [13:0] a;
        input [15:0] expected;
        begin
            @(negedge clk);
            we   = 1'b0;
            addr = a;
            @(posedge clk);
            #1;
            if (rdata !== expected) begin
                $display("STORE MISMATCH slot=%0d got=0x%04h expected=0x%04h",
                         a, rdata, expected);
                errors = errors + 1;
            end
        end
    endtask

    initial begin
        clk    = 1'b0;
        we     = 1'b0;
        addr   = 14'd0;
        wdata  = 16'd0;
        errors = 0;

        // Write: slot k holds the code point that maps to it, U+AC00 + k.
        for (k = 0; k < 11172; k = k + 1) begin
            @(negedge clk);
            we    = 1'b1;
            addr  = k[13:0];
            wdata = 16'hAC00 + k[15:0];
        end
        @(negedge clk);
        we = 1'b0;

        // Read back every slot.
        for (k = 0; k < 11172; k = k + 1)
            check_slot(k[13:0], 16'hAC00 + k[15:0]);

        // Reserved address: a write is ignored and a read returns zero.
        @(negedge clk);
        we    = 1'b1;
        addr  = 14'd11172;
        wdata = 16'hFFFF;
        @(negedge clk);
        we   = 1'b0;
        addr = 14'd11172;
        @(posedge clk);
        #1;
        if (rdata !== 16'h0000) begin
            $display("STORE MISMATCH reserved slot 11172 got=0x%04h expected 0x0000", rdata);
            errors = errors + 1;
        end

        @(negedge clk);
        we    = 1'b1;
        addr  = 14'd16383;
        wdata = 16'hFFFF;
        @(negedge clk);
        we   = 1'b0;
        addr = 14'd16383;
        @(posedge clk);
        #1;
        if (rdata !== 16'h0000) begin
            $display("STORE MISMATCH reserved slot 16383 got=0x%04h expected 0x0000", rdata);
            errors = errors + 1;
        end

        if (errors == 0)
            $display("PASS: all 11,172 slots verified, reserved addresses read zero");
        else
            $display("FAIL: %0d mismatches", errors);

        $finish;
    end
endmodule
