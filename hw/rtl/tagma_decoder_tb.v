// Exhaustive testbench for tagma_decoder.
//
// Verifies all 11,172 valid code points (0xAC00 .. 0xD7A3) plus the upper
// boundary. Two modes:
//
//   default:      expected values are recomputed from the formula inside
//                 the testbench (make sim)
//   GOLDEN_CHECK: expected values come from hw/rtl/golden_anchors.hex,
//                 exported from the tagma_core reference engine
//                 (make sim-golden)
//
// The simulation command lives in hw/Makefile.

`timescale 1ns/1ps

module tagma_decoder_tb;
    reg  [15:0] code;
    wire [4:0]  i;
    wire [4:0]  m;
    wire [4:0]  f;

    integer errors;
    integer k;
`ifndef GOLDEN_CHECK
    reg  [13:0] offset;
    reg  [13:0] exp_i;
    reg  [13:0] exp_m;
    reg  [13:0] exp_f;
`endif

`ifdef GOLDEN_CHECK
    reg [28:0] golden [0:11171];
`endif

    tagma_decoder dut (
        .code(code),
        .i(i),
        .m(m),
        .f(f)
    );

    initial begin
        errors = 0;

`ifdef TRACE
        // VCD activity trace for power estimation (make sim-trace).
        $dumpfile("trace.vcd");
        $dumpvars(0, dut);
`endif

`ifdef GOLDEN_CHECK
        $readmemh("golden_anchors.hex", golden);

        // All 11,172 entries exported from the reference engine.
        for (k = 0; k < 11172; k = k + 1) begin
            code = 16'hAC00 + {2'b00, golden[k][28:15]};
            #1;
            if (i !== golden[k][14:10] || m !== golden[k][9:5] || f !== golden[k][4:0]) begin
                $display("GOLDEN MISMATCH line=%0d offset=0x%04h got i=%0d m=%0d f=%0d expected i=%0d m=%0d f=%0d",
                         k, golden[k][28:15], i, m, f,
                         golden[k][14:10], golden[k][9:5], golden[k][4:0]);
                errors = errors + 1;
            end
        end
`else
        // All 11,172 valid syllables: 0xAC00 + k for k in [0, 11171].
        for (k = 0; k < 11172; k = k + 1) begin
            code   = 16'hAC00 + k[15:0];
            offset = k[13:0];
            exp_i  = offset / 14'd588;
            exp_m  = (offset % 14'd588) / 14'd28;
            exp_f  = offset % 14'd28;
            #1;
            if (i !== exp_i[4:0] || m !== exp_m[4:0] || f !== exp_f[4:0]) begin
                $display("MISMATCH code=0x%04h offset=%0d got i=%0d m=%0d f=%0d expected i=%0d m=%0d f=%0d",
                         code, offset, i, m, f, exp_i, exp_m, exp_f);
                errors = errors + 1;
            end
        end
`endif

        // Upper boundary: last syllable (0xD7A3) -> i=18, m=20, f=27.
        // Note: the Hangul block ends at 0xD7A3, not 0xD7AF; 0xAC00 + 11171.
        code = 16'hD7A3;
        #1;
        if (i !== 5'd18 || m !== 5'd20 || f !== 5'd27) begin
            $display("MISMATCH at upper boundary");
            errors = errors + 1;
        end

        if (errors == 0) begin
`ifdef GOLDEN_CHECK
            $display("PASS: all 11,172 golden anchors verified");
`else
            $display("PASS: all 11,172 code points verified");
`endif
        end else
            $display("FAIL: %0d mismatches", errors);

        $finish;
    end
endmodule
