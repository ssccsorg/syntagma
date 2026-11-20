// SPDX-License-Identifier: CERN-OHL-S-2.0
// Copyright (C) 2026 Taeho Lee
//
// This source describes Open Hardware and is licensed under the CERN-OHL-P v2.
// You may redistribute and modify this documentation and make products using
// it under the terms of the CERN-OHL-P v2 (https://cern.ch/cern-ohl). This
// documentation is distributed WITHOUT ANY EXPRESS OR IMPLIED WARRANTY,
// INCLUDING OF MERCHANTABILITY, SATISFACTORY QUALITY AND FITNESS FOR A
// PARTICULAR PURPOSE. Please see the CERN-OHL-P v2 for applicable conditions.
//
// The anchors, compared one coordinate at a time on the die.
//
// tagma_volume_verify_top sums the packs the decoder reports and compares the
// total with a constant the host computed. A sum cannot say which coordinate
// was wrong, and two coordinates that trade value keep the total: a swap of two
// packs is invisible to it. This top reads the same anchor lines the reference
// wrote, takes the pack each line already carries, and compares it with the
// pack the decoder produces for that line's code point. The verdict is that
// comparison, so a single wrong coordinate is the same size as eleven thousand.
//
// The ROM is the anchor file itself: one 32-bit line a record, offset in
// [28:15] and the axes in [14:0], which is the layout hw/rtl/golden_anchors.hex
// uses. The offset is the field the reference wrote, so the die decodes the code
// point that line names and nothing is recomputed on the host side.
//
// The path is fixed for synthesis because $readmemh resolves against the
// synthesis working directory, which is hw/synth/yosys for this flow. A
// simulation takes the file from `+rom=`, so one top checks a fixture and a
// whole anchor file without a rebuild.
//
//   stage[0] -> N4,  iCELink PA0,  driven by the host
//   stage[1] -> J5,  iCELink PA1,  driven by the host
//   pin_match -> H5, iCELink PA13, read by the host
//   led_green -> A11, led_blue -> A12, led_red -> B11, the RGB channels
//
// pin_match is set only when a full pass finished with no mismatch and no line
// whose offset is outside the alphabet. COUNT is supplied by the flow from the
// file it reads, so the two cannot disagree.

module tagma_anchor_verify_top #(
    parameter integer COUNT = 0
) (
    input  wire clk_in,
    input  wire [1:0] stage,
    output wire pin_match,
    output wire led_red,
    output wire led_green,
    output wire led_blue
);
    // The alphabet the decoder covers: U+AC00 plus this many offsets.
    localparam [15:0] COORD_BASE  = 16'hAC00;
    localparam [15:0] COORD_RADIX = 16'd11172;

    // The ROM read is one stage and the decoder is four, so a line takes five
    // clocks to reach the comparison.
    localparam integer LATENCY = 5;
    localparam integer ADDR_BITS = (COUNT > 1) ? $clog2(COUNT) : 1;
    // The ROM is addressed by an ADDR_BITS counter, so its depth is that whole
    // address space rather than COUNT. A depth that is not a power of two leaves
    // the mapping to round, and a memory that comes up short reads as a mismatch
    // on every line past the shortfall. COUNT still governs the wrap and the
    // comparison, and no address at or above it is ever read.
    localparam integer DEPTH = 1 << ADDR_BITS;

    wire clk;

`ifdef SIM
    assign clk = clk_in;
`else
    pipe_pll pll (
        .clk_in(clk_in),
        .clk(clk),
        .locked()
    );
`endif

    reg [31:0] rom [0:DEPTH-1];

`ifdef SIM
    reg [1023:0] rom_path;
    initial begin
        if (!$value$plusargs("rom=%s", rom_path))
            rom_path = "out/anchor_rom.hex";
        $readmemh(rom_path, rom);
    end
`else
    initial $readmemh("out/anchor_rom.hex", rom);
`endif

    reg [ADDR_BITS-1:0] addr = 0;

    always @(posedge clk)
        addr <= (addr == COUNT - 1) ? {ADDR_BITS{1'b0}} : addr + 1'b1;

    // Stage 0: the ROM read. Synchronous, because an asynchronous read of this
    // size would be distributed logic rather than a block RAM.
    reg [31:0] line;

    always @(posedge clk)
        line <= rom[addr];

    wire [13:0] offset = line[28:15];
    wire [15:0] code = COORD_BASE + {2'b00, offset};

    wire [4:0] pi, pm, pf;

    tagma_decoder_pipe dut (
        .clk(clk),
        .code(code),
        .i(pi),
        .m(pm),
        .f(pf)
    );

    // The line the pipeline is answering is delayed to meet its output, so the
    // expected pack compared here is the one that named this code point. Only
    // [28:0] is read downstream, so the chain carries that much.
    reg [28:0] l1, l2, l3, l4;

    always @(posedge clk) begin
        l1 <= line[28:0];
        l2 <= l1;
        l3 <= l2;
        l4 <= l3;
    end

    wire [14:0] pack = {pi, pm, pf};
    wire [13:0] offset_d = l4[28:15];
    wire [14:0] expected = l4[14:0];

    // The pipeline fills for four clocks after reset, so the first LATENCY
    // comparisons are not real. A shift register opens the window once.
    reg [LATENCY-1:0] fill = {LATENCY{1'b0}};

    always @(posedge clk)
        if (!fill[LATENCY-1])
            fill <= {fill[LATENCY-2:0], 1'b1};

    wire ready = fill[LATENCY-1];

    // A line whose offset is outside the alphabet is counted and not compared:
    // the reference has no axes for a code point it cannot name.
    wire outside = offset_d >= COORD_RADIX;
    wire differs = pack != expected;

    reg [31:0] mismatches = 32'd0;
    reg [31:0] outside_count = 32'd0;
    reg [ADDR_BITS-1:0] seen = 0;

    wire [31:0] mismatch_next =
        (!outside && differs) ? mismatches + 32'd1 : mismatches;
    wire [31:0] outside_next =
        outside ? outside_count + 32'd1 : outside_count;

    reg matched = 1'b0;

    always @(posedge clk) begin
        if (ready) begin
            if (seen == COUNT - 1) begin
                // This line closes the pass, so the next counters hold the
                // whole file's totals. Neither count is cleared on the closing
                // edge: a pass that found a mismatch has to keep saying so.
                matched <= (mismatch_next == 32'd0) && (outside_next == 32'd0);
                mismatches <= 32'd0;
                outside_count <= 32'd0;
                seen <= {ADDR_BITS{1'b0}};
            end else begin
                mismatches <= mismatch_next;
                outside_count <= outside_next;
                seen <= seen + 1'b1;
            end
        end
    end

    assign pin_match = matched;

    // The stage selects a colour. Stage 3 is the verdict, and there the die's
    // answer decides between green and red, so the light is not the host's
    // opinion of the run.
    reg red, green, blue;

    always @* begin
        case (stage)
            2'd0:    {red, green, blue} = 3'b001;
            2'd1:    {red, green, blue} = 3'b011;
            2'd2:    {red, green, blue} = 3'b110;
            2'd3:    {red, green, blue} = matched ? 3'b010 : 3'b100;
            default: {red, green, blue} = 3'b000;
        endcase
    end

    // A part that sits still reads as a photograph of a part. The colour it was
    // asked for blinks until the verdict, and the verdict is steady.
    reg [24:0] heart = 25'd0;

    always @(posedge clk)
        heart <= heart + 1'b1;

    wire beat = heart[24];
    wire hold = stage == 2'd3;

    assign led_red   = red   && (beat || hold);
    assign led_green = green && (beat || hold);
    assign led_blue  = blue  && (beat || hold);
endmodule
