# Tagma decoder synthesis report

Tool versions: Yosys 0.65 (git sha1 aec814bdf3071f7e0fd0fbe43f7f711e99d01e24), Verilator 5.048.

## Verification

`make sim` runs the exhaustive testbench through Verilator:

```
PASS: all 11,172 code points verified
```

All valid code points in [0xAC00, 0xD7A3] plus the upper boundary check (i=18, m=20, f=27) pass. The upper boundary is 0xD7A3, not 0xD7AF: the Hangul block contains exactly 11,172 syllables, and 0xAC00 + 11,171 = 0xD7A3.

## Generic synthesis (ev-compatible schema)

Flow: `read_verilog -sv; hierarchy; proc; synth; stat -json`, identical to `ev/src/synth/backends/yosys.rs`. The JSON report (`synth/yosys/reports/generic_stat.json`) feeds directly into the `SynthesisMetrics` schema with `num_cells = 206`.

| Metric | Value |
|--------|-------|
| num_cells | 206 |
| cell mix | ANDNOT 13, AND 47, MUX 17, NAND 61, NOR 5, NOT 7, ORNOT 9, OR 18, XNOR 19, XOR 10 |
| registers | 0 |
| memories | 0 |

## Gate-level estimate

Flow: `proc; flatten; opt; techmap; opt; abc -g AND,NAND,OR,NOR,XOR,XNOR; stat` (2-input gate library).

| Metric | Value |
|--------|-------|
| cells | 232 |
| cell mix | AND 57, NAND 98, NOR 9, NOT 15, OR 21, XNOR 23, XOR 9 |

This is an estimate over a generic 2-input library, not a PDK standard cell library. The OpenROAD flow in Phase 4 replaces it with a Sky130 report.

## iCE40 FPGA target

Flow: `synth_ice40 -top tagma_decoder -json out/tagma_decoder_ice40.json`.

| Resource | Count |
|----------|-------|
| SB_LUT4 | 95 |
| SB_CARRY | 66 |
| SB_DFF | 0 |

The design fits any iCE40 device. Place and route (`nextpnr-ice40` + `icepack`) runs in Phase 3 once the demo board is chosen.

## Post-synthesis verification

The synthesized netlist (`out/tagma_decoder_gates.v`) is verified on top of the RTL checks:

| Check | Result |
|-------|--------|
| Gate-level simulation against golden anchors (Verilator) | PASS: all 11,172 golden anchors verified |
| Formal equivalence RTL vs gate netlist (`equiv.ys`) | proven, 50 cells, all 2^16 inputs |
| VCD activity trace (`make sim-trace`) | trace.vcd emitted for power estimation |

Formal equivalence is stronger than simulation: `equiv_simple` and `equiv_induct` prove the RTL and the synthesized structure are logically identical over every possible input, so the gate count and the functional result are tied together.

## Interpretation

The measured gate counts (206 cells generic, 232 cells 2-input gate estimate) are below the ~300 gate claim in the Tagma whitepaper. The claim holds and is conservative for this decoder structure: pure combinational logic, zero registers, constant divisors mapped to shift-subtract networks. Timing closure and the standard cell number must still be demonstrated with a real PDK flow; that is Phase 4.

## Reproduction

```bash
make -C hw check    # sim + sim-golden + golden-check + gatesim + equiv + synth
make -C hw sim-trace   # VCD activity trace for power estimation
```
