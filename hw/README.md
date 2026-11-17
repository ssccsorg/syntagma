# Tagma hardware verification

The code is a monotone packing of the coordinate axes, so a range query over them is an addition rather than a tree search: the bounds of a prefix range come out of the code, and the iteration is a stride. That is the property the hardware is for, and it follows from the formula rather than from any measurement here.

This tree turns the "~300 gates, 1 cycle" claim into verifiable artifacts: an exhaustive RTL verification against the reference coordinate engine, an FPGA place-and-route whose bitstream is checked against the RTL, and a standard cell synthesis report. Three units make up the coordinate primitive in hardware: the decoder (code point to axes), the compose unit (axes to index, the decoder's inverse), and the distance unit (field-wise absolute difference of two coordinates). Decode and distance take a 16-bit code point, while compose returns the coordinate index, the offset from U+AC00, so its code point is U+AC00 + index. That is the split the Rust `Coord` uses.

## Status

| Item | State |
|------|-------|
| `rtl/tagma_decoder.v` 3-axis combinational decoder | Implemented |
| `rtl/tagma_decoder_tb.v` exhaustive testbench (11,172 code points) | Implemented, passing |
| `rtl/tagma_segment_store.v` 11,172 x 16-bit segment store (behavioral model) | Implemented, passing |
| `rtl/tagma_segment_store_tb.v` exhaustive testbench (11,172 slots, reserved addresses, read-during-write) | Implemented, passing |
| `rtl/tagma_compose.v` axis-to-index compose (decoder inverse) | Implemented, passing |
| `rtl/tagma_compose_tb.v` exhaustive testbench (32^3 combinations, golden) | Implemented, passing |
| `rtl/tagma_dist.v` field-wise distance of two coordinates (two decoders) | Implemented, passing |
| `rtl/tagma_dist_tb.v` exhaustive testbench (11,172 code points, varied operands, golden) | Implemented, passing |
| Golden-anchor cross-check against `tagma_core` (Rust reference) | Implemented, passing |
| `tools/check_golden_anchors.py` consistency gate | Implemented, passing |
| Gate-level netlist simulation against golden anchors (decoder, compose, distance) | Implemented, passing |
| Formal equivalence: RTL vs gate netlist (decoder, compose, distance) | Proven |
| `rtl/demo/tagma_demo_top_tb.v` demo top, axes on the 11,172 valid syllables and the validity LED on all 65,536 code points | Implemented, passing |
| Bitstream channel: the placed-and-routed design recovered with `icebox_vlog` and compared against the RTL over all 65,536 code points (`make sim-pnr`) | Implemented, passing |
| FPGA demo top + Upduino 3.1 PCF + PnR flow | Implemented, bitstream, 16.79 MHz Fmax |
| iCESugar-Pro v1.3 (ECP5) bring-up bitstream | Implemented, programs over the module iCELink, 344 MHz Fmax |
| iCESugar-Pro v1.3 (ECP5) tagma demo bitstream | Implemented, 33 IO, 22.28 ns worst path at 25 MHz, needs external wiring |
| Streaming decoder, four stages, one code point per clock | Implemented, `make sim-pipe` passes over four sweeps, 148.77 MHz register to register on the ECP5, ceiling set by the multiplier |
| Self-checking streaming bitstream (pipeline against the reference, 40 MHz) | Implemented, 56.62 MHz register to register, 51.96 MHz on the reference path; the reference is now the slower unit |
| On-device verification image (sweep checksum against the tagma_core anchors) | Implemented, `pin_match` reads 1 at 40, 100, 120, 125 and 140 MHz, a wrong constant reads 0, 150 MHz is rejected at 145.99 |
| Per-value comparison of the same anchors (gate `make sim-anchor`, image `DESIGN=anchor`) | Implemented; the gate compares all 11,172 lines one at a time, answering 1 for the anchor file and 0 for the same file with two packs exchanged, both files carrying the sum `065BEA26`, and the die image built from the same pair answers both verdicts at 40 MHz |
| iCELink GPIO channel, ECP5 F5/H5/N4/J5 to PA14/PA13/PA0/PA1 | Implemented, read on all four, drive on two (N4 and J5; PA14 is the SWD clock), PA13 is read only |
| Decoder optimization (multiply-shift) | Implemented, meets 12 MHz board clock |
| Software reference bench (`../sw/rust/benches/bench_hw.rs`) | Implemented, results in comments |
| Yosys generic synthesis report (`stat -json`, ev-compatible schema) | Committed at `synth/yosys/reports/generic_stat.json` |
| Yosys gate-level estimate (2-input gate library) | `synth/yosys/reports/gates_stat.txt`, plus `compose_gates_stat.txt` and `dist_gates_stat.txt` |
| Yosys iCE40 synthesis report (LUT count) | `synth/yosys/reports/ice40_stat.txt` |
| OpenRAM chton SRAM configuration | Draft, requires OpenRAM + PDK |
| FPGA board demo (physical) | Bitstream programs and boots on the iCESugar-Pro; the switch and LED wiring that makes it a decoder demonstration is the remaining step |
| OpenROAD standard cell report (Sky130) | Implemented, measured locally + CI (hw job) |
| Power at the workload's activity, from a trace as well as from a hand model, as the report step's `POST_FINAL_REPORT_TCL` hook (`openroad/designs/sky130hd/tagma_demo/power_activity.tcl`) | Implemented, runs in the `hw` job: 57.8 uW at the tool default, 36.6 uW modelled, 36.1 uW from the trace, about 3.0 pJ per decode at 12 MHz; the absolute figure is provisional |

## Layout

- `boards/` per-board constraints, one directory per physical board
- `rtl/` the primitive: the decoder, the pipeline, the composer, the distance unit, the segment store, their testbenches, and `golden_anchors.hex`
- `rtl/demo/` the board tops and diagnostic images; nothing here is part of a claim about the primitive
- `demo/` the harness that records a run of a board image, and one package per demonstration; `demo/FORMAT.md` states the video format, and each package narrows it in its own `FORMAT.md`
- `synth/yosys/` Yosys synthesis scripts, the PnR flow, and the committed reports
- `openroad/` OpenROAD standard cell flow (Sky130)
- `openram/` OpenRAM configuration for the chton segment store
- `tools/` golden anchor consistency gate

## Boards

Each physical board owns one directory under `boards/`, and `make pnr BOARD=<board>` selects both its constraints and its toolchain. The two boards share the RTL and nothing else.

| Board | Device | Constraints | Clock | Flow |
|---|---|---|---|---|
| `upduino31` | iCE40UP5K-SG48 | `boards/upduino31/tagma_demo.pcf` | 12 MHz | `synth_ice40`, `nextpnr-ice40`, `icepack` |
| `icesugar_pro` | ECP5 LFE5U-25F-6BG256C | `boards/icesugar_pro/bringup.lpf` | 25 MHz | `synth_ecp5`, `nextpnr-ecp5`, `ecppack` |

`nextpnr-ecp5` is not packaged by Homebrew, so the Pro flow needs OSS CAD Suite on `PATH`; `ecppack` comes from `prjtrellis`.

The Pro carries two debuggers when it sits in its Ext-Board: the module's iCELink and the Ext-Board's own DAPLink. Only the module's iCELink reaches the ECP5. It enumerates as `0x1d50:0x602b` with a volume named `iCELink`; the Ext-Board DAPLink enumerates as `0x0d28:0x0204` with `DAPLINK`, and its JTAG scan returns all ones. Programming writes the bitstream to flash:

```sh
icesprog -w hw/synth/yosys/out/bringup.bit
```

`icesprog` is built from `tools/src` in the icesugar repository. The SRAM path through the shipped `blink_sram.svf` fails its final status check on this board, so flash is the programming route.

The demo runs on the Pro as `DESIGN=tagma_demo`. It needs 16 switch inputs and 15 LED outputs wired to the Ext-Board headers; `boards/icesugar_pro/README.md` holds the wiring table and the bank and voltage notes. The board alone can show only the validity bit, on the RGB LED.

Three further designs run on the Pro without external wiring. `DESIGN=stream` reports a register-to-register clock for the pipelined decoder, `DESIGN=stream_check` compares that pipeline against the combinational unit and reports on the RGB balls, and `DESIGN=verify VERIFY_MHZ=<rate>` accumulates a checksum over the valid range, compares it on the die against a constant derived from the golden anchors, and leaves the verdict on three pins the iCELink reaches as GPIO. `make board-verify` reads those pins back over the programming cable, so the verdict and the PLL rate are measured without a person at the board.

## Why 16-bit: the enabling conditions

The whole hardware design rests on four independent conditions, of which BMP membership is one:

1. The Hangul syllable block U+AC00..U+D7A3 sits in the Unicode BMP, so one syllable is one UTF-16 code unit, one `u16`.
2. The Unicode Hangul syllable composition algorithm decomposes every syllable into initial 19 x medial 21 x final 28 axes. This property is independent of the BMP.
3. The valid range is contiguous (0xAC00 + k for k in [0, 11172)), which makes the single-offset arithmetic possible.
4. The Unicode stability policy keeps the block fixed, so the constants (0xAC00, 0xD7A3, 11172, 588, 28) are durable.

The general Tagma mechanism does not require the BMP; multi-unit and astral cases use the N-dimensional machinery (`CoordPath`, `base11172`, `CoordSpace2`). Hangul is the cheapest demonstration of the method, which is why the decoder is small enough to fit the ~300 gate claim.

## Verification

Each target answers a different question, and the numbers they produce are in the status table.

| Target | What it establishes |
|---|---|
| `make sim` | the decoder against the formula over all 11,172 valid code points |
| `make sim-golden` | the same points against `rtl/golden_anchors.hex`, which `tagma_core` exports |
| `make golden-check` | the generated anchor file against the decomposition contract |
| `make sim-pipe` | the four-stage decoder against the reference over four sweeps |
| `make sim-store`, `make sim-compose`, `make sim-dist` | the segment store, the composer, and the distance unit over their input spaces |
| `make sim-demo` | the board top: the axes on the valid syllables, the validity bit on all 65,536 code points |
| `make sim-pnr` | the placed-and-routed logic recovered with `icebox_vlog`, against the RTL over all 65,536 code points |
| `make gatesim`, `make equiv` | the synthesized netlist against the anchors, and the RTL against the netlist |
| `make synth`, `make report` | Yosys and OpenROAD synthesis, writing the committed reports |
| `make pnr BOARD=<board>` | place and route for one board from the same RTL |
| `make board-verify` | the on-device sweep, read back over the programming cable |

`make check` runs every row except `pnr`, `board-verify`, and `report`, which need a board or a standard cell library.

`make check` needs Verilator, Yosys, and the iCE40 tools (`nextpnr-ice40`, `icepack`, `icetime`, `icebox_vlog`). `run.sh` skips the gate and names the missing tool when one is absent.

## License

The hardware design in this directory is licensed under the CERN-OHL-P v2, the
permissive variant of the CERN Open Hardware Licence version 2. The full text is
in [LICENSE](LICENSE). Covered source is the HDL under `rtl/`, the synthesis and
place-and-route scripts under `synth/`, `openroad/`, and `openram/`, the board
constraints, and the verification tooling under `tools/`.

The software reference implementation under `../sw/` stays under the Apache License
2.0, the repository default.

Files here carried no per-file notice before this section existed, so the root
Apache License 2.0 applied to them. That grant to earlier recipients is not
revoked; this tree is under CERN-OHL-P v2 from here on.

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

