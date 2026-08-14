# Tagma hardware verification

This tree turns the "~300 gates, 1 cycle" claim into verifiable artifacts: an exhaustive RTL verification against the reference coordinate engine, an FPGA synthesis report, and a standard cell synthesis report.

## Status

| Item | State |
|------|-------|
| `rtl/tagma_decoder.v` 3-axis combinational decoder | Implemented |
| `rtl/tagma_decoder_tb.v` exhaustive testbench (11,172 code points) | Implemented, passing |
| Yosys generic synthesis report (`stat -json`, ev-compatible schema) | Report in `docs/` |
| Yosys gate-level estimate (2-input gate library) | Report in `docs/` |
| Yosys iCE40 synthesis report (LUT count) | Report in `docs/` |
| OpenRAM chton SRAM configuration | Draft, requires OpenRAM + PDK |
| FPGA board demo (iCE40/ECP5 or Artix-7) | Next phase |
| OpenROAD standard cell report (Sky130) | Next phase |

## Layout

- `rtl/` Verilog sources and testbench
- `synth/yosys/` Yosys synthesis scripts and generated reports
- `openram/` OpenRAM configuration for the chton segment store
- `docs/` synthesis and verification reports

## Development plan

### Phase 1: RTL, exhaustive simulation, synthesis reports

This phase runs on branch `48-hw-verification` (issue #48).

1. `tagma_decoder.v`: pure combinational decoder, offset decomposition `offset = code - 0xAC00`, `i = offset / 588`, `m = (offset % 588) / 28`, `f = offset % 28`.
2. `tagma_decoder_tb.v`: exhaustive check of all 11,172 valid syllables plus the upper boundary, driven by Verilator.
3. Synthesis scripts that mirror the canonical Yosys invocation in `ev/src/synth/backends/yosys.rs` (`read_verilog -sv; hierarchy; proc; synth; stat -json`) so the metrics schema stays compatible with `SynthesisMetrics` (num_cells to gate_count, area). A second script maps to a 2-input gate library for the gate count behind the claim. A third script targets iCE40 for the LUT count.
4. CI hook: `hw/Makefile` with `sim`, `synth`, `check` targets, wired into `run.sh` and skipped when the tools are absent.

### Phase 2: Golden-anchor cross-validation against the reference engine

The Phase 1 testbench recomputes the same formula it checks, which is self-referential. Phase 2 removes that risk by generating golden vectors from the reference coordinate engine in `sw/rust` and feeding them to the RTL testbench, following the golden-anchor pattern in `ssccs/poc/baremetal_riscv/sv` (`_golden_anchors.svh` plus `check_golden_anchors.py`). Deliverables: a golden vector file (offset, i, m, f) exported from the Rust core, a testbench mode that reads it, and a consistency gate that fails when RTL and reference diverge.

### Phase 3: FPGA board demo

Synthesize the decoder with an FPGA top module (switches for the 16-bit code point, LEDs for i/m/f) and place and route with the open flow: Yosys to JSON, `nextpnr` to bitstream, `icepack` for iCE40. The demo target is a low-cost iCE40 or ECP5 board so the whole toolchain stays open source; an Artix-7 (Vivado) flow is the fallback. Deliverables: constraints file, bitstream, board bring-up notes, and a demo video.

### Phase 4: chton SRAM and standard cell report

Generate the chton segment store SRAM with OpenRAM (`openram/chton_sram.py`, SkyWater 130nm, 11,172 x 16-bit single port). Run the standard cell flow with OpenROAD: synthesis, placement and routing, timing and power report. Compare the reported gate count against the ~300 gate claim and publish the number.

## Integration with existing SSCCS tooling

- `ev/src/synth/backends/yosys.rs` is the canonical Yosys integration: subprocess invocation, `stat -json` parsing, `SynthesisMetrics` with `gate_count` and `cell_area`. The scripts in `synth/yosys/` follow the same flow, so the numbers are directly comparable and can be emitted as neXus `Fact` envelopes (`fact_type: synthesis_result`, `origin: ev/synthesis/yosys`) later.
- `ssccs/poc/baremetal_riscv/sv` establishes the Verilator plus Makefile plus golden-anchor verification pattern. `hw/Makefile` mirrors that pattern, and Phase 2 adopts the golden-anchor gate.

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
