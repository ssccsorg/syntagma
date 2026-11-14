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
// On-device verification of the coordinate values a volume carries.
//
// tagma_pipe_verify_top checks the decoder against the reference anchors, which
// is a statement about tagma_core's valid range. This top makes the same check
// over the values an actual store wrote: the coordinate values of a volume's
// identifiers are supplied as a ROM, so what the die confirms is that the
// addresses on that medium are coordinates, computed by a decoder that did not
// write them.
//
// The ROM is `out/volume_rom.hex`, one 16-bit value a line, in the order the
// host emits them. The path is fixed because `$readmemh` resolves against the
// synthesis working directory, which is `hw/synth/yosys` for this flow. The
// flow reads the same file to set COUNT, so the two cannot disagree.
//
// A value below COORD_RADIX is a coordinate. Anything else is counted as
// invalid and kept out of the sum, so a single corrupted value changes the
// checksum and raises the invalid count, and either one clears pin_match.
//
// The verdict leaves the die on the pin the debugger reads, and the stage the
// host sets decides the colour of the on-board RGB LED, so a person filming the
// board sees which step the run is on rather than only its end.
//
//   stage[0] -> F5,  iCELink PA14, driven by the host
//   stage[1] -> N4,  iCELink PA0,  driven by the host
//   stage[2] -> J5,  iCELink PA1,  driven by the host
//   pin_match -> H5, iCELink PA13, read by the host (the firmware keeps this one)
//   led_green -> A11, led_blue -> A12, led_red -> B11, the RGB channels
//
// Stage 4 is the verdict, and the colour it shows is the die's answer rather
// than the host's: green when the sum and the alphabet agree, red otherwise. A
// host that asks for the verdict and gets red has been told something.
//
// EXPECTED must be supplied by the flow. The default of zero never matches a
// non-empty volume, so an unset parameter reads as a failure.

module tagma_volume_verify_top #(
    parameter [31:0] EXPECTED = 32'h00000000,
    parameter integer COUNT = 0
) (
    input  wire clk_in,
    input  wire [2:0] stage,
    output wire pin_match,
    output wire led_red,
    output wire led_green,
    output wire led_blue
);
    // The alphabet the engine writes: U+AC00 plus this many values.
    localparam [15:0] COORD_RADIX = 16'd11172;

    // The ROM read is one stage and the decoder is four, so a value takes five
    // clocks to reach the accumulator.
    localparam integer LATENCY = 5;
    localparam integer ADDR_BITS = (COUNT > 1) ? $clog2(COUNT) : 1;

    wire clk;

`ifdef SIM
    // A simulation has no PLL primitive, so the clock is the input.
    assign clk = clk_in;
`else
    pipe_pll pll (
        .clk_in(clk_in),
        .clk(clk),
        .locked()
    );
`endif

    reg [15:0] rom [0:COUNT-1];
    initial $readmemh("out/volume_rom.hex", rom);

    reg [ADDR_BITS-1:0] addr = 0;

    always @(posedge clk)
        addr <= (addr == COUNT - 1) ? {ADDR_BITS{1'b0}} : addr + 1'b1;

    // Stage 0: the ROM read. The array is synchronous because an asynchronous
    // read of this size would be distributed logic rather than a block RAM.
    reg [15:0] value;

    always @(posedge clk)
        value <= rom[addr];

    // The ROM holds the coordinate value, which is 0..11171, and the decoder
    // takes the Unicode code point it names. The base is added once here so the
    // value on the medium and the value in the ROM stay the same number.
    wire [15:0] code = value + 16'hAC00;

    wire [4:0] pi, pm, pf;

    tagma_decoder_pipe dut (
        .clk(clk),
        .code(code),
        .i(pi),
        .m(pm),
        .f(pf)
    );

    // The accumulator works on the pack the pipeline reports, so the value that
    // produced it is delayed to meet it at the same edge.
    reg [15:0] v1, v2, v3, v4;

    always @(posedge clk) begin
        v1 <= value;
        v2 <= v1;
        v3 <= v2;
        v4 <= v3;
    end

    // The pipeline fills for four clocks after reset, so the first LATENCY
    // packs are not real. A shift register opens the window once.
    reg [LATENCY-1:0] fill = {LATENCY{1'b0}};

    always @(posedge clk)
        if (!fill[LATENCY-1])
            fill <= {fill[LATENCY-2:0], 1'b1};

    wire ready = fill[LATENCY-1];

    // pack = (i << 10) | (m << 5) | f, the packing the anchors use.
    wire [14:0] pack = {pi, pm, pf};

    reg [31:0] acc = 32'd0;
    reg [31:0] invalids = 32'd0;
    reg [ADDR_BITS-1:0] seen = 0;

    wire valid = v4 < COORD_RADIX;
    wire [31:0] acc_next = valid ? acc + {17'd0, pack} : acc;
    wire [31:0] invalid_next = valid ? invalids : invalids + 32'd1;

    reg        matched = 1'b0;

    always @(posedge clk) begin
        // Nothing the pipeline reports before the fill completes belongs to a
        // ROM value, so it is dropped rather than counted.
        if (ready) begin
            if (seen == COUNT - 1) begin
                // This pack closes the pass, so acc_next and invalid_next are
                // the whole volume's totals. The invalid count is not cleared
                // here: it is the verdict, and a pass that saw a value outside
                // the alphabet has to keep saying so after it closes.
                matched  <= (acc_next == EXPECTED) && (invalid_next == 32'd0);
                acc      <= 32'd0;
                seen     <= {ADDR_BITS{1'b0}};
            end else begin
                acc      <= acc_next;
                invalids <= invalid_next;
                seen     <= seen + 1'b1;
            end
        end
    end

    assign pin_match = matched;

    // The stage selects a colour. Stage 4 is the verdict, and there the die's
    // answer decides between green and red, so the light is not the host's
    // opinion of the run.
    reg red, green, blue;

    always @* begin
        case (stage)
            3'd0:    {red, green, blue} = 3'b001;  // writing the volume
            3'd1:    {red, green, blue} = 3'b011;  // reaching a record
            3'd2:    {red, green, blue} = 3'b110;  // what an answer costs
            3'd3:    {red, green, blue} = 3'b101;  // programming the part
            3'd4:    {red, green, blue} = matched ? 3'b010 : 3'b100;
            default: {red, green, blue} = 3'b000;
        endcase
    end

    assign led_red   = red;
    assign led_green = green;
    assign led_blue  = blue;
endmodule
