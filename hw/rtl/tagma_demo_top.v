// Tagma decoder FPGA demo top module.
//
// Targets the Upduino 3.1 (iCE40UP5K-SG48). The 16-bit code point comes
// in on GPIO switches, the three axes drive LED groups, and the onboard
// green LED lights when the code point is a valid Hangul syllable
// (0xAC00..0xD7A3). The decode is combinational; the outputs are
// registered on the board clock so the one-cycle latency and the timing
// closure are measurable with a real clock.
//
// The onboard LED is negative logic (write 0 to light), so led_valid
// carries the inverted validity signal. Pin mapping:
// hw/synth/yosys/upduino31_demo.pcf

module tagma_demo_top (
    input  wire        clk,
    input  wire [15:0] code,
    output reg  [4:0]  i,
    output reg  [4:0]  m,
    output reg  [4:0]  f,
    output wire        led_valid
);
    wire [4:0] i_c;
    wire [4:0] m_c;
    wire [4:0] f_c;
    reg  valid;  // internal validity flag, drives the negative-logic LED

    tagma_decoder dec (
        .code(code),
        .i(i_c),
        .m(m_c),
        .f(f_c)
    );

    always @(posedge clk) begin
        i     <= i_c;
        m     <= m_c;
        f     <= f_c;
        valid <= (code >= 16'hAC00) && (code <= 16'hD7A3);
    end

    // Negative-logic LED: light it when the code point is valid.
    assign led_valid = ~valid;
endmodule
