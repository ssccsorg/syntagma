# Tagma decoder synthesis report

Tool versions: Yosys 0.65 (git sha1 aec814bdf3071f7e0fd0fbe43f7f711e99d01e24), Verilator 5.048.

## Verification

`make sim` runs the exhaustive testbench through Verilator:

```
PASS: all 11,172 code points verified
```

All valid code points in [0xAC00, 0xD7A3] plus the upper boundary check (i=18, m=20, f=27) pass. The upper boundary is 0xD7A3, not 0xD7AF: the Hangul block contains exactly 11,172 syllables, and 0xAC00 + 11,171 = 0xD7A3. The same vectors pass against the Rust reference (golden mode) and against the synthesized gate netlist, and formal equivalence proves the RTL and the netlist identical over all 2^16 inputs.

## Design trade-off: naive vs multiply-shift

Two implementations of the same arithmetic were measured. The naive shift-subtract divider matches the ~300 gate claim but does not close timing on a 12 MHz board. The multiply-shift network costs more gates and meets the clock; the demo bitstream uses this version.

| Metric | shift-subtract (naive) | multiply-shift (current) |
|--------|------------------------|--------------------------|
| generic cells (ev schema) | 206 | 478 |
| gate-level estimate | 232 | 588 |
| iCE40 LUT4 + SB_CARRY | 95 + 66 | 201 + 41 |
| PnR ICESTORM_LC | 177 | 255 |
| logic levels | 72 | 33 |
| critical path (UP5K) | 115.90 ns | 59.55 ns |
| Fmax | 8.63 MHz | 16.79 MHz |
| meets 12 MHz board clock | no | yes |

## Generic synthesis (ev-compatible schema)

Flow: `read_verilog -sv; hierarchy; proc; synth; stat -json`, identical to `ev/src/synth/backends/yosys.rs`. The JSON report (`hw/synth/yosys/reports/generic_stat.json`) feeds directly into the `SynthesisMetrics` schema with `num_cells = 478`.

| Metric | Value |
|--------|-------|
| num_cells | 478 |
| cell mix | ANDNOT 25, AND 82, MUX 15, NAND 135, NOR 9, NOT 6, ORNOT 15, OR 44, XNOR 76, XOR 71 |
| registers | 0 |
| memories | 0 |

## Gate-level estimate

Flow: `proc; flatten; opt; techmap; opt; abc -g AND,NAND,OR,NOR,XOR,XNOR; stat` (2-input gate library).

| Metric | Value |
|--------|-------|
| cells | 588 |
| cell mix | AND 113, NAND 192, NOR 29, NOT 23, OR 44, XNOR 84, XOR 103 |

This is an estimate over a generic 2-input library, not a PDK standard cell library. The OpenROAD flow in Phase 4 replaces it with a Sky130 report.

## iCE40 FPGA target

Flow: `synth_ice40 -top tagma_decoder -json hw/synth/yosys/out/tagma_decoder_ice40.json`.

| Resource | Count |
|----------|-------|
| SB_LUT4 | 201 |
| SB_CARRY | 41 |
| SB_DFF | 0 |

## Post-synthesis verification

The synthesized netlist (`hw/synth/yosys/out/tagma_decoder_gates.v`) is verified on top of the RTL checks:

| Check | Result |
|-------|--------|
| Gate-level simulation against golden anchors (Verilator) | PASS: all 11,172 golden anchors verified |
| Formal equivalence RTL vs gate netlist (`equiv.ys`) | proven, 50 cells, all 2^16 inputs |
| VCD activity trace (`make sim-trace`) | trace.vcd emitted for power estimation |

Formal equivalence is stronger than simulation: `equiv_simple` and `equiv_induct` prove the RTL and the synthesized structure are logically identical over every possible input, so the gate count and the functional result are tied together.

## FPGA place and route (demo target)

Target: Upduino 3.1 (iCE40UP5K-SG48), 12 MHz onboard clock, registered demo top (`hw/rtl/tagma_demo_top.v`, constraints in `hw/synth/yosys/upduino31_demo.pcf`). Flow: `hw/synth/yosys/synth_demo.ys` -> `nextpnr-ice40` -> `icepack` -> `icetime` (`hw/synth/yosys/run_pnr.sh`).

| Metric | Value |
|--------|-------|
| ICESTORM_LC | 255 / 5280 (4%) |
| SB_IO | 33 / 39 (84%) |
| SB_GB | 1 / 8 |
| critical path (icetime) | 59.55 ns |
| Fmax | 16.79 MHz |
| logic levels | 33 |

The multiply-shift network closes the 12 MHz board clock with margin (16.79 MHz measured). The bitstream is generated and the design is ready for board bring-up.

## Software reference baseline

`sw/rust/benches/bench_hw.rs` measures the reference implementation in criterion style (bench.rs convention), with the key results documented in comments:

| Bench | Value |
|-------|-------|
| hw/decode/single | 1.44 ns |
| hw/decode/all_11172 | 1.92 µs |
| hw/decode/index_all_11172 (baseline) | 0.457 µs |
| hw/golden/export_11172 | 4.56 µs |

Run with `cargo bench --manifest-path sw/rust/Cargo.toml --bench bench_hw`.

## Interpretation

The ~300 gate claim in the Tagma whitepaper holds for the naive shift-subtract decoder (206 to 232 cells). Timing closure on a 12 MHz board requires the multiply-shift structure (478 to 588 cells), which is the version the demo uses. The one-cycle latency holds for both. A real PDK standard cell number must still be demonstrated with the OpenROAD flow; that is Phase 4.

## Reproduction

```bash
make -C hw check    # sim + sim-golden + golden-check + gatesim + equiv + synth
make -C hw sim-trace   # VCD activity trace for power estimation
bash hw/synth/yosys/run_pnr.sh   # demo PnR: bitstream + icetime timing report
```
