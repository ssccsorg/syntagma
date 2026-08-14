// Exhaustive testbench for tagma_decoder.
//
// Verifies all 11,172 valid code points (0xAC00 .. 0xD7A3) plus the upper
// boundary. The simulation command lives in hw/Makefile (make sim).

`timescale 1ns/1ps

module tagma_decoder_tb;
    reg  [15:0] code;
    wire [4:0]  i;
    wire [4:0]  m;
    wire [4:0]  f;

    integer errors;
    integer k;
    reg  [13:0] offset;
    reg  [13:0] exp_i;
    reg  [13:0] exp_m;
    reg  [13:0] exp_f;

    tagma_decoder dut (
        .code(code),
        .i(i),
        .m(m),
        .f(f)
    );

    initial begin
        errors = 0;

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

        // Upper boundary: last syllable (0xD7A3) -> i=18, m=20, f=27.
        // Note: the Hangul block ends at 0xD7A3, not 0xD7AF; 0xAC00 + 11171.
        code = 16'hD7A3;
        #1;
        if (i !== 5'd18 || m !== 5'd20 || f !== 5'd27) begin
            $display("MISMATCH at upper boundary");
            errors = errors + 1;
        end

        if (errors == 0)
            $display("PASS: all 11,172 code points verified");
        else
            $display("FAIL: %0d mismatches", errors);

        $finish;
    end
endmodule
