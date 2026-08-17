# Tagma hardware verification: RTL, golden-anchor cross-check, synthesis reports

Status report for issue #48 on branch `48-hw-verification`. This entry records the purpose of the hardware verification work, what was delivered in the first two phases, and what becomes possible once the remaining phases land.

## Purpose

The Tagma whitepaper claims the 3-axis decoder fits in about 300 gates and decodes in one cycle. A claim is not evidence. The goal of the hardware track is to replace the claim with verifiable artifacts: an exhaustive RTL verification against the reference coordinate engine, an FPGA synthesis report, and eventually a physical board demo and a standard cell synthesis report. The same RTL and memory configuration are the first step toward propagating the Tagma coordinate space into chton as a physical SRAM segment store.

## What was delivered

### Phase 1: RTL, exhaustive simulation, synthesis reports

- `hw/rtl/tagma_decoder.v`: pure combinational 3-axis decoder. `offset = code - 0xAC00`, `i = offset / 588`, `m = (offset % 588) / 28`, `f = offset % 28`. No registers, no divider macros; constant divisors map to shift-subtract networks.
- `hw/rtl/tagma_decoder_tb.v`: exhaustive testbench over all 11,172 valid code points plus the upper boundary, driven by Verilator.
- `hw/synth/yosys/`: three flows. The generic flow mirrors the canonical invocation in `ev/src/synth/backends/yosys.rs` (`proc; synth; stat -json`) so the metrics stay compatible with `SynthesisMetrics`. The gate-level flow maps to a 2-input gate library. The iCE40 flow reports LUT utilization for the FPGA target.
- `hw/openram/chton_sram.py`: OpenRAM configuration for the chton segment store (11,172 x 16-bit, single port, SkyWater 130nm). Draft pending OpenRAM and PDK availability.
- `hw/Makefile` and a `check_hw` hook in `run.sh`: the hardware gate runs in CI when Verilator and Yosys are installed and is skipped otherwise.

### Phase 2: Golden-anchor cross-validation against the reference engine

The Phase 1 testbench recomputed the same formula it checks, which is self-referential. Phase 2 removes that risk:

- `sw/rust/core/examples/golden_export.rs`: exports the full coordinate decomposition from the `tagma_core` reference engine through the `Coord::to_axes` path into `hw/rtl/golden_anchors.hex`. Each line is one packed 29-bit value: offset[28:15], i[14:10], m[9:5], f[4:0].
- `hw/rtl/tagma_decoder_tb.v`: golden mode (`make sim-golden`) reads the anchor file and checks the decoder against the reference vectors.
- `hw/tools/check_golden_anchors.py`: consistency gate over the generated anchor file, validating the format and the decomposition contract on all 11,172 entries. The anchor file itself is a build artifact produced by `make golden-export` before each golden simulation; the exporter is the committed source of truth. The pattern follows `ssccs/poc/baremetal_riscv/sv` (`_golden_anchors.svh` plus `check_golden_anchors.py`).

### Post-synthesis verification

The synthesized netlist is verified on top of the RTL checks:

- `hw/synth/yosys/equiv.ys` plus the `gatesim` and `equiv` Makefile targets: the gate-level netlist (`out/tagma_decoder_gates.v`) is simulated against the golden anchors through Verilator, and formal equivalence between the RTL and the netlist is proven with `equiv_simple` and `equiv_induct` over all 2^16 inputs.
- `make sim-trace` emits a VCD activity trace (`trace.vcd`) for later power estimation in the OpenROAD flow.

### Phase 3: FPGA demo flow

The demo top module (`hw/rtl/tagma_demo_top.v`) maps the 16-bit code point from switches to three LED groups and lights the onboard green LED for valid syllables, with outputs registered on the board clock. Constraints use real Upduino 3.1 pins (verified against the FPGAwars/icestudio reference PCF). The open flow runs end to end (`hw/synth/yosys/run_pnr.sh`): `synth_demo.ys` to JSON, `nextpnr-ice40` to ASC, `icepack` to bitstream, `icetime` to timing.

### Decoder optimization (multiply-shift)

The first PnR run measured 115.90 ns (8.63 MHz) with 72 logic levels: the shift-subtract divider depth missed the 12 MHz board clock. The decoder was reimplemented with multiply-shift constant division, exact over the valid domain (`(q4 * 9363) >> 16` for division by 28, `(j * 781) >> 14` for division by 21). The full verification suite re-passed unchanged (exhaustive, golden, gate-level, formal equivalence), confirming the two networks are functionally identical. Measured after optimization: 255 ICESTORM_LC (4%), 33 logic levels, 59.55 ns, Fmax 16.79 MHz, which closes the 12 MHz clock with margin. The gate count rose from 206 to 478 cells (generic schema) as the cost of timing closure. Physical board bring-up and the demo video remain.

### Phase 4: OpenROAD standard cell flow

The Sky130 standard cell flow is set up under `hw/openroad/` with ORFS, the official OpenROAD Flow Scripts image. The registered demo top is the design target so the report maps one to one to the FPGA demo; the pure decoder gate count is reported separately with `yosys stat -liberty` against the Sky130 cell library. The flow runs in the `p4` CI job on the x86 runner and uploads the reports as an artifact.

Local finding: the ORFS image is x86_64 only. On Apple Silicon under Rosetta the flow reaches placement and then crashes in CTS with an illegal instruction, an OpenROAD emulation limitation. Phase 4 therefore executes natively in CI rather than on the ARM host. The first CI run fetches the Sky130 PDK automatically.

### Bench center

`sw/rust/benches/bench_hw.rs` follows the `bench.rs` convention: criterion microbenchmarks with the key results documented in comments, measured on this host. It provides the software reference baseline for the hardware numbers: single decode 1.44 ns, full-space decode 1.92 µs, index-only baseline 0.457 µs, golden export 4.56 µs. Registered in `sw/rust/benches/Cargo.toml` and wired into `run.sh --bench`.

## Verification results

| Check | Result |
|-------|--------|
| Exhaustive simulation, formula mode | PASS: all 11,172 code points verified |
| Exhaustive simulation, golden mode | PASS: all 11,172 golden anchors verified |
| Golden anchor consistency gate | golden anchors OK: 11172 entries |
| Gate-level netlist simulation against golden anchors | PASS: all 11,172 golden anchors verified |
| Formal equivalence RTL vs netlist | proven, 50 cells, all 2^16 inputs |
| Generic synthesis (`stat -json`, ev schema) | 478 cells, 0 registers |
| Gate-level estimate (2-input library) | 588 cells |
| iCE40 synthesis | 201 LUT4 + 41 SB_CARRY, 0 registers |
| PnR (UP5K, Upduino 3.1 demo) | 255 ICESTORM_LC, Fmax 16.79 MHz, 59.55 ns critical path |
| Design trade-off | shift-subtract 206..232 cells / 8.63 MHz vs multiply-shift 478..588 cells / 16.79 MHz |
| Software reference (bench_hw) | single 1.44 ns, all 1.92 µs, export 4.56 µs |

The ~300 gate claim holds for the naive shift-subtract structure (206 to 232 cells); the multiply-shift version used by the demo exceeds it (478 to 588 cells) as the cost of closing the 12 MHz board clock. Timing closure and a real PDK standard cell number still need the OpenROAD flow.

## A boundary correction found by exhaustive testing

The exhaustive test exposed that the last valid Hangul syllable is U+D7A3, not U+D7AF. The Unicode block contains exactly 11,172 syllables and 0xAC00 + 11,171 = 0xD7A3. Inputs in U+D7A4..U+D7AF decode to an out-of-range initial axis and are structurally invalid; the `tagma_core` reference documents these as filler positions. The RTL valid range comment and the testbench boundary were corrected accordingly. Any document or spec that states the range as ending at 0xD7AF should be checked against this.

## What becomes possible after this work

- Phase 3 (FPGA board demo): the demo flow runs end to end on the Upduino 3.1 target and the decoder now closes the 12 MHz board clock (255 ICESTORM_LC, Fmax 16.79 MHz, bitstream produced). The remaining step is physical board bring-up and the demo video. The 8 MB PSRAM on the Upduino also covers the RAM-attached test device question.
- Phase 4 (standard cell report and chton propagation): OpenRAM generates the chton segment store SRAM from `hw/openram/chton_sram.py`; OpenROAD then produces the timing, area, and power numbers for a real PDK. That report is the artifact that finally substantiates the gate count on silicon rather than in a generic library.
- ev and neXus integration: the generic synthesis report already uses the `stat -json` schema of `ev/src/synth/backends/yosys.rs`, so the numbers can be emitted as neXus `Fact` envelopes (`fact_type: synthesis_result`) without a schema change.
- Cross-language verification: the golden anchor file is a language-neutral contract. The same file can gate a C++ implementation (`sw/cpp`) later, and the exporter pattern extends to any future implementation of the coordinate engine.

## References

- syntagma issue #48: hw: Tagma decoder RTL, FPGA verification, and standard cell synthesis report, <https://github.com/ssccsorg/syntagma/issues/48>
- syntagma branch `48-hw-verification`
- Tagma whitepaper: <https://doi.org/10.5281/zenodo.21302508>
- SSCCS whitepaper: <https://doi.org/10.5281/zenodo.18759106>
- ev Yosys backend: `ev/src/synth/backends/yosys.rs`
- poc golden anchor pattern: `ssccs/poc/baremetal_riscv/sv`
- OpenRAM: <https://github.com/VLSIDA/OpenRAM>
- OpenROAD: <https://github.com/The-OpenROAD-Project/OpenROAD>
