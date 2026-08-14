// Tagma 3-axis combinational decoder.
//
// Input:  16-bit Hangul syllable code point in [0xAC00, 0xD7A3]
// Output: i choseong index (0..18)
//         m jungseong index (0..20)
//         f jongseong index (0..27)
//
// Mapping (Tagma whitepaper, https://doi.org/10.5281/zenodo.21302508):
//   offset = code - 0xAC00
//   i = offset / 588          (588 = 21 * 28)
//   m = (offset % 588) / 28
//   f = offset % 28
//
// The module is a pure combinational path with no registers, so the
// latency is exactly one cycle. Constant divisors map to shift-subtract
// networks at synthesis; no divider macro is instantiated.

module tagma_decoder (
    input  wire [15:0] code,
    output wire [4:0]  i,
    output wire [4:0]  m,
    output wire [4:0]  f
);
    // offset in [0, 11171], fits in 14 bits.
    // Inputs below 0xAC00 underflow and are outside the valid domain.
    /* verilator lint_off UNUSEDSIGNAL */
    wire [15:0] sub = code - 16'hAC00;
    wire [13:0] offset = sub[13:0];

    // All intermediates are 14 bits wide; the high bits of the outputs
    // are unused by construction (valid input domain), so Verilator's
    // UNUSEDSIGNAL lint is disabled for this block.
    wire [13:0] j  = offset / 14'd28;        // offset / 28 in [0, 398]
    wire [13:0] ii = j / 14'd21;             // offset / 588 in [0, 18]
    wire [13:0] mm = j - ii * 14'd21;        // (offset % 588) / 28 in [0, 20]
    wire [13:0] ff = offset - j * 14'd28;    // offset % 28 in [0, 27]
    /* verilator lint_on UNUSEDSIGNAL */

    assign i = ii[4:0];
    assign m = mm[4:0];
    assign f = ff[4:0];
endmodule
