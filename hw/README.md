# Tagma hardware verification

This tree turns the "~300 gates, 1 cycle" claim into verifiable artifacts: an exhaustive RTL verification against the reference coordinate engine, an FPGA synthesis report, and a standard cell synthesis report.

## Status

| Item | State |
|------|-------|
| `rtl/tagma_decoder.v` 3-axis combinational decoder | Implemented |
| `rtl/tagma_decoder_tb.v` exhaustive testbench (11,172 code points) | Implemented, passing |
| Golden-anchor cross-check against `tagma_core` (Rust reference) | Implemented, passing |
| `tools/check_golden_anchors.py` consistency gate | Implemented, passing |
| Gate-level netlist simulation against golden anchors | Implemented, passing |
| Formal equivalence: RTL vs gate netlist | Proven |
| FPGA demo top + Upduino 3.1 PCF + PnR flow | Implemented, bitstream + timing report |
| Software reference bench (`sw/rust/benches/bench_hw.rs`) | Implemented, results in comments |
| Yosys generic synthesis report (`stat -json`, ev-compatible schema) | Report in `docs/` |
| Yosys gate-level estimate (2-input gate library) | Report in `docs/` |
| Yosys iCE40 synthesis report (LUT count) | Report in `docs/` |
| OpenRAM chton SRAM configuration | Draft, requires OpenRAM + PDK |
| FPGA board demo (physical) | Next, board required |
| OpenROAD standard cell report (Sky130) | Next phase |

## Layout

- `rtl/` Verilog sources and testbench
- `synth/yosys/` Yosys synthesis scripts, PnR flow, and generated reports
- `openram/` OpenRAM configuration for the chton segment store
- `tools/` golden anchor consistency gate
- `docs/` synthesis and verification reports

## Development plan

### Phase 1: RTL, exhaustive simulation, synthesis reports

This phase runs on branch `48-hw-verification` (issue #48).

1. `tagma_decoder.v`: pure combinational decoder, offset decomposition `offset = code - 0xAC00`, `i = offset / 588`, `m = (offset % 588) / 28`, `f = offset % 28`.
2. `tagma_decoder_tb.v`: exhaustive check of all 11,172 valid syllables plus the upper boundary, driven by Verilator.
3. Synthesis scripts that mirror the canonical Yosys invocation in `ev/src/synth/backends/yosys.rs` (`read_verilog -sv; hierarchy; proc; synth; stat -json`) so the metrics schema stays compatible with `SynthesisMetrics` (num_cells to gate_count, area). A second script maps to a 2-input gate library for the gate count behind the claim. A third script targets iCE40 for the LUT count.
4. CI hook: `hw/Makefile` with `sim`, `synth`, `check` targets, wired into `run.sh` and skipped when the tools are absent.

### Phase 2: Golden-anchor cross-validation against the reference engine

The Phase 1 testbench recomputes the same formula it checks, which is self-referential. Phase 2 removes that risk: `sw/rust/core/examples/golden_export.rs` exports the full coordinate decomposition from the `tagma_core` reference engine (the `Coord::to_axes` path) into `rtl/golden_anchors.hex`, one packed 29-bit value per line (offset[28:15] i[14:10] m[9:5] f[4:0]). The testbench golden mode (`make sim-golden`) reads that file and checks the decoder against the reference vectors. `tools/check_golden_anchors.py` gates the file format and the decomposition contract, following the golden-anchor pattern in `ssccs/poc/baremetal_riscv/sv`.

Delivered: golden exporter, golden testbench mode, Python consistency gate, all wired into `make check` and `run.sh`. The anchor file is a generated build artifact (`make golden-export`) regenerated before each golden simulation; the exporter is the committed source of truth, and the gate validates the generated file against the decomposition contract.

### Post-synthesis verification

The synthesized netlist is verified on top of the RTL checks: the gate-level netlist is simulated against the golden anchors (`make gatesim`) and formal equivalence between the RTL and the netlist is proven over all 2^16 inputs (`make equiv`, `synth/yosys/equiv.ys`). `make sim-trace` emits a VCD activity trace for the later power estimation step.

### Phase 3: FPGA board demo

The demo top module (`rtl/tagma_demo_top.v`, switches for the 16-bit code point, LED groups for i/m/f, onboard green LED for validity) and the Upduino 3.1 constraints (`synth/yosys/upduino31_demo.pcf`) are in place. The open flow runs end to end: `synth_demo.ys` to JSON, `nextpnr-ice40` to ASC, `icepack` to bitstream, `icetime` to timing (`synth/yosys/run_pnr.sh`). Measured on the UP5K: 177 ICESTORM_LC (3%), critical path 115.90 ns, Fmax 8.63 MHz; the design does not meet the 12 MHz board clock, which is the documented target for the next optimization (multiply-shift constant division). Remaining: physical board bring-up and demo video.

### Phase 4: chton SRAM and standard cell report

Generate the chton segment store SRAM with OpenRAM (`openram/chton_sram.py`, SkyWater 130nm, 11,172 x 16-bit single port). Run the standard cell flow with OpenROAD: synthesis, placement and routing, timing and power report. Compare the reported gate count against the ~300 gate claim and publish the number.

## Software reference baseline

The decoder is also benchmarked in software against the hardware numbers. `sw/rust/benches/bench_hw.rs` follows the `bench.rs` convention (criterion, key results in comments): single decode 1.44 ns, full-space decode 1.92 µs, index-only baseline 0.457 µs, golden export 4.56 µs. Run with `cargo bench --manifest-path sw/rust/Cargo.toml --bench bench_hw`.

## Integration with existing SSCCS tooling

- `ev/src/synth/backends/yosys.rs` is the canonical Yosys integration: subprocess invocation, `stat -json` parsing, `SynthesisMetrics` with `gate_count` and `cell_area`. The scripts in `synth/yosys/` follow the same flow, so the numbers are directly comparable and can be emitted as neXus `Fact` envelopes (`fact_type: synthesis_result`, `origin: ev/synthesis/yosys`) later.
- `ssccs/poc/baremetal_riscv/sv` establishes the Verilator plus Makefile plus golden-anchor verification pattern. `hw/Makefile` mirrors that pattern, and Phase 2 adopts the golden-anchor gate.

## Why 16-bit: the enabling conditions

The whole hardware design rests on four independent conditions, of which BMP membership is one:

1. The Hangul syllable block U+AC00..U+D7A3 sits in the Unicode BMP, so one syllable is one UTF-16 code unit, one `u16`.
2. The Unicode Hangul syllable composition algorithm decomposes every syllable into initial 19 x medial 21 x final 28 axes. This property is independent of the BMP.
3. The valid range is contiguous (0xAC00 + k for k in [0, 11172)), which makes the single-offset arithmetic possible.
4. The Unicode stability policy keeps the block fixed, so the constants (0xAC00, 0xD7A3, 11172, 588, 28) are durable.

The general Tagma mechanism does not require the BMP; multi-unit and astral cases use the N-dimensional machinery (`CoordPath`, `base11172`, `CoordSpace2`). Hangul is the cheapest demonstration of the method, which is why the decoder is small enough to fit the ~300 gate claim.

## References

- Tagma whitepaper: https://doi.org/10.5281/zenodo.21302508
- SSCCS whitepaper: https://doi.org/10.5281/zenodo.18759106
- SSCCS documentation: https://docs.ssccs.org/
- OpenRAM: https://github.com/VLSIDA/OpenRAM
- Yosys: https://github.com/YosysHQ/yosys
- nextpnr: https://github.com/YosysHQ/nextpnr
- icestorm (iCE40 bitstream tools): https://github.com/YosysHQ/icestorm
- OpenROAD: https://github.com/The-OpenROAD-Project/OpenROAD
- SkyWater 130nm PDK: https://github.com/google/skywater-pdk
- ev verification CLI: https://github.com/ssccsorg/ev
- poc reference designs: https://github.com/ssccsorg/poc

## Agent handoff prompt

```
Task: Implement the Tagma 3-axis decoder in Verilog, verify it exhaustively, and
produce synthesis reports for FPGA and gate-level targets.

Input specification:
- 16-bit Hangul code point in [0xAC00, 0xD7A3]
- Outputs: i in [0, 18], m in [0, 20], f in [0, 27]
- offset = code - 0xAC00; i = offset / 588; m = (offset % 588) / 28; f = offset % 28
- Target: ~300 gates, one cycle, pure combinational

Deliverables:
1. hw/rtl/tagma_decoder.v
2. hw/rtl/tagma_decoder_tb.v with exhaustive validation of all 11,172 code points
3. hw/synth/yosys/: generic, gate-level, and iCE40 synthesis scripts with reports
4. hw/docs/synthesis_report.md with real tool output
5. hw/openram/chton_sram.py for the chton segment store (11,172 x 16-bit SRAM)
6. hw/Makefile with sim/synth/check targets, wired into run.sh

Constraints:
- Mirror the Yosys flow in ev/src/synth/backends/yosys.rs (stat -json schema)
- Follow the Verilator plus Makefile pattern in ssccs/poc/baremetal_riscv/sv
- English only, no em dashes, no emojis

References:
- Tagma whitepaper: https://doi.org/10.5281/zenodo.21302508
- OpenRAM: https://github.com/VLSIDA/OpenRAM
- Target FPGA: iCE40 (open flow) or Artix-7 (Vivado fallback)
```
