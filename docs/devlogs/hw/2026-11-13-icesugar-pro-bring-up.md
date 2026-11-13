# iCESugar-Pro bring-up: a second board, two debuggers, and the demo on ECP5

Status report for issue #75 on branch `75-icesugar-pro`. The board split and the demo port landed under #75 and #76, and the two were folded into one issue and one branch once the work turned out to be a single subject. This entry records the board arrival, the diagnosis that was wrong before it was right, the board split, and what the demo does on an ECP5 today.

## Purpose

The hardware track documented a demo for the Upduino 3.1, and no such board was on hand. The physical demonstration was recorded as pending. An iCESugar-Pro v1.3 arrived, so the same RTL now has a device to run on, and the multi-target step in the hardware document ("port the demo flow to ECP5 or Artix-7") has a board choice.

The finding to test was the document's own claim that the hardware is not a speedup for the dense N=1 case. A second device is not there to make that case differently; it is there to check that the primitive's realization is substrate data rather than a one-board artifact.

## The board and its two debuggers

The iCESugar-Pro module carries an iCELink. The Ext-Board it sits in carries its own DAPLink, inherited from the Colorlight i5 carrier it was designed for. Both enumerate at once, and only one of them reaches the ECP5.

| Debugger | USB ID | Volume | Firmware | Reaches the ECP5 |
|---|---|---|---|---|
| module iCELink | `0x1d50:0x602b` | `iCELink` | iCELink by MuseLab, 2021-01-05, interface 0255 | yes |
| Ext-Board DAPLink | `0x0d28:0x0204` | `DAPLINK` | DAPLink by MuseLab, 2020-10-03, interface 0254 | no |

The Ext-Board DAPLink answers the CMSIS-DAP handshake and returns `0xffffffff` for every JTAG IDCODE read. It also lacks the iCELink vendor protocol: `CMD_SYS_GET_INFO` (0x80) and `CMD_SYS_JTAG_SEL` (0x84) return a one-byte DAP status, and the tool's read of the following bytes returns its own sent opcode, which is why a board type read as `BT_iCESugar` and a flash probe read `0x0`. Those are not board facts.

On the module iCELink, `icesprog` reports `board: [iCESugar-Pro]` and `flash: [w25q256] (32MB)`, the ECP5 IDCODE reads `0x41111043`, and a flash write reads back byte-identical.

## What the first diagnosis got wrong

The session spent time on a firmware theory before the port was questioned. The record matters because the symptoms were convincing.

- A 104 KB iCE40 bitstream was dropped onto the DAPLink volume and was consumed without `FAIL.TXT`, which suggested the flash had been overwritten. It had not: the design kept running, and the target was the wrong debugger.
- The Pro's 582 KB `.bit` timed out with `error: The transfer timed out`. The Ext-Board DAPLink could not reach the ECP5.
- Advice to reflash the iCELink firmware by shorting RST to GND was checked against the schematic and rejected: `BOOT0` is tied to GND through a 10K resistor, there is no reset button, and the 128 KB image would cover the bootloader. Nothing was reflashed.
- The SRAM SVF path shipped in the demo fails its final status check on this board (`SIR 3C`, DONE not set, error bits 23 and 25). Flash programming works, so flash is the route.

The decisive fact was the schematic README: the Ext-Board has its own DAPLink and the Pro "doesn't need it". After moving the cable to the module's own Type-C port, every symptom disappeared at once.

## Board split (issue #75)

Board constraints moved under `boards/<board>/` and `make pnr BOARD=<board>` selects both the constraints and the toolchain.

| Board | Device | Constraints | Clock | Flow |
|---|---|---|---|---|
| `upduino31` | iCE40UP5K-SG48 | `boards/upduino31/tagma_demo.pcf` | 12 MHz | `synth_ice40`, `nextpnr-ice40`, `icepack` |
| `icesugar_pro` | ECP5 LFE5U-25F-6BG256C | `boards/icesugar_pro/bringup.lpf` | 25 MHz | `synth_ecp5`, `nextpnr-ecp5`, `ecppack` |

`run_pnr.sh` holds one complete flow per board, and neither branch reads the other board's constraints. CI path filters collapsed to `hw/boards/**`. The Upduino result is unchanged after the move: 255 ICESTORM_LC, 59.55 ns, 16.79 MHz, 33 logic levels, and `sim-pnr` still agrees with the RTL over all 65,536 code points.

The Pro bring-up design drives the onboard RGB LED from three counter bits, so the new flow had a design that needs no external wiring. It reports 33 logic elements, a 344 MHz Fmax against a 25 MHz clock, and a 582,369 byte bitstream with the `LFE5U-25F-6CABGA256` header.

## Demo port (issue #76)

`boards/icesugar_pro/tagma_demo.lpf` maps 33 signals. Header labels came from `doc/iCESugar-pro-pinmap.png` and banks from `doc/LFE5U-25F-5BG256-Pinout.csv`.

The first mapping failed with `incompatible IO voltages 2V5 and 3V3 on bank 1`, because `m[4]` (ball `B8`) shares bank 1 with the RGB LED. Bank 1 runs at 2.5V. The mapping now keeps one standard per bank:

| Bank | Pins | Standard | Signals |
|---|---|---|---|
| 1 | A12, B8 | `LVCMOS25` | `led_valid`, `f[4]` |
| 6, 8 | P3 | `LVCMOS33` | `code[15:0]` |
| 0, 7 | P2 | `LVCMOS33` | `i`, `m`, `f[0..3]` |
| 6 | P6 | `LVCMOS33` | 25 MHz clock |

Each signal group sits on one header: all 16 usable pins of P3 carry `code`, and 15 of the 20 pins of P2 carry the axis outputs. The wiring table is in `boards/icesugar_pro/README.md`.

The `code` inputs are pulled up. A passive switch leaves its pin floating when open, so without the pull the value is indeterminate both unwired and with a DIP module. An unwired P3 now reads `0xFFFF`, which is invalid.

## Polarity is board data

The demo ran and drove the RGB LED, which was the first hardware evidence that the design was alive. The observed state was the blue channel lit while `code` was `0xFFFF`, that is, while the code point was invalid.

Two facts follow. A12 is the blue channel, not the green one the constraints had assumed, and the RGB LED lights on a high level. The RTL drove `led_valid = ~valid`, which suits the Upduino's negative-logic LED and showed the opposite state here.

`tagma_demo_top` now takes `LED_ACTIVE_LOW`, defaulting to 1 so the Upduino path and the simulation are unchanged. The Pro flow overrides it to 0 with `chparam`. This replaces the fixed inversion recorded in the earlier engineering decisions table; the polarity is a property of the board, like the clock pin.

## Measured results

| Metric | Upduino 3.1 (UP5K) | iCESugar-Pro (ECP5 25F) |
|---|---|---|
| Logic | ICESTORM_LC 255 / 5280 | TRELLIS_COMB 50 / 24288, TRELLIS_FF 16 / 24288 |
| I/O | SB_IO 33 / 39 | TRELLIS_IO 33 / 197 |
| Multipliers | not reported | MULT18X18D 4 / 28 |
| Critical path | 59.55 ns | 22.85 ns |
| Clock constraint | 12 MHz | 25 MHz |
| Fmax | 16.79 MHz | not reported, no interior register-to-register path |

The two paths are not the same quantity. The 59.55 ns figure is the whole critical path that `icetime` reports for the placed design; the 22.85 ns figure is the worst cross-domain path `nextpnr-ecp5` reports for an input to a register, because this design has no register-to-register path to constrain. The numbers are recorded as two device results, not as a speedup.

## Verified and pending

Verified on hardware: the bitstream builds, `icesprog -w` writes it, a 4096-byte read-back matches the bitstream, `PROGRAMN` on `PB7` reloads it, the JTAG IDCODE stays `0x41111043`, and the design boots and drives the RGB LED.

Pending: the post-fix observation (the LED should be dark on the invalid `0xFFFF`), the one-wire check (`code[14]` to GND forms `0xBFFF`, a valid syllable, and should light it), the full 16-switch and 15-LED wiring, and the demo capture. Until the wiring exists, the demo is verified as a build and boot, not as a decoder demonstration.

## Next

The hardware document's next-steps table needs three corrections, because it still says the demo target is the Upduino 3.1 and that the physical demonstration waits for a board. The measured-results table gains the ECP5 column with the comparability note above.

After the wiring closes the physical track, the streaming step is the next RTL subject: a pipelined decoder at one code point per clock, which is the case the memory document identifies as the acceleration opportunity. The segment store also has a new option, since the Pro carries 32 MB of SDRAM and can exercise the address map on a device while the OpenRAM macro waits for a PDK.

## The streaming track

Throughput is the case where hardware has an argument, and the combinational decoder does not have one: it costs a full multiply chain per code point. `tagma_decoder_pipe.v` keeps the same arithmetic and the same constants, split at the multiplier boundary so each stage carries one multiplier.

| Stage | Work |
|---|---|
| 1 | `offset` to `q4`, `r2`, and `j = (q4 * 9363) >> 16` |
| 2 | `i = (j * 781) >> 14`, `m = j - i * 21`, `f = 4 * (q4 - j * 7) + r2` |

Latency is two edges and throughput is one code point per clock. `tagma_decoder_pipe_tb.v` drives the valid range continuously for four full sweeps and compares every emitted triple against the combinational decoder, which the matrix, golden-anchor, and recovered-netlist channels already pin at 11,172 of 11,172 points. An exact match at every cycle after the fill therefore carries that evidence to the pipeline, and the absence of idle cycles is the throughput claim.

```
PASS: 44689 outputs checked over 44691 clocks, 4 full sweeps of the valid space, one code point per clock
```

## What place and route reports for the pipeline alone

`tagma_pipe_top.v` sources `code` from a counter, so every path runs register to register through the decoder. That is what the earlier section could not get: the demo takes `code` from pins, so all its paths were cross-domain and no clock was reported.

| Metric | Value |
|---|---|
| Fmax at a 25 MHz constraint | 77.69 MHz |
| TRELLIS_COMB | 39 |
| TRELLIS_FF | 43 |
| MULT18X18D | 4 |
| CCU2C | 10 |
| LUT4 | 11 |
| Bitstream | 582,369 bytes |

The critical path is inside stage 2, not stage 1: `j_q` into the multiply by 781, out as `ii`, into the multiply by 21, out as `mm`, at 12.18 ns. Splitting stage 2 again would be the next gain, and the arithmetic allows it because `m` and `f` need `j` and `i` only.

## Self-check on silicon

A bitstream that drives header pins with no wiring behind it proves only that it programmed. `tagma_pipe_check_top.v` instead runs the pipeline and the combinational reference from the same counter, delays the reference by the pipeline depth, and compares the two triples at every clock. Any mismatch latches an error indication, so a dark error channel after minutes of running is evidence that the two agree at one code point per clock on the device.

Some care went into the rate. `pipe_pll` doubles the 25 MHz oscillator, but the reference decoder sets the ceiling: at a 50 MHz constraint its path measured 19.48 ns, which is 51.34 MHz and 2.6 percent of margin. The PLL is therefore set to 40 MHz, where the same path has about 27 percent. The check still runs the pipeline above the board clock, which is the point.

| Metric | Value |
|---|---|
| Clock | 40 MHz from a 25 MHz input, VCO 600 MHz |
| Fmax, register to register | 53.70 MHz, PASS at 40 MHz |
| Fmax, reference path | 50.63 MHz, PASS at 40 MHz |
| TRELLIS_COMB | 185 |
| TRELLIS_FF | 120 |
| MULT18X18D | 7 |
| EHXPLLL | 1 |

The three observations are named by function and constrained by ball, because the vendor README gives the RGB balls as a set without a colour per ball.

| Signal | Ball | Meaning |
|---|---|---|
| `led_clk` | A11 | blinks at 2.4 Hz while the fast clock runs |
| `led_err` | A12 | lights and latches on any mismatch |
| `led_swept` | B11 | lights once 2^28 clocks have been compared |

`led_swept` is a rate reading rather than a completion flag, because the comparison saturates and never clears. `2^28` clocks is 6.71 s at 40 MHz and 10.74 s at 25 MHz, so the interval between configuration and that channel lighting separates the two clocks with a stopwatch and no instrument.

The first build compared against `28'hFFFFFF`, six hex digits against a 28-bit register. The comparison type-checked and the design built, but the count saturated at 2^24 clocks and the readout fired after 0.42 s. A literal and a register that disagree in width still compare, so the elapsed-time claim has to be checked against the constant rather than against the width of the variable.

## The ECP5 drops a mixed register initializer

The check top started `code` at `16'hAC00` so the first comparison cycles would already be in the valid range. The synthesized netlist does not carry that value. Every `TRELLIS_FF` in the `code` register has no `INIT` parameter, `LATTICE_GSR` reports only `Handling GSR in tagma_pipe_check_top`, and nothing is warned.

A four-line control confirms the behaviour is general rather than a property of this design:

```verilog
module inittest (input wire clk, output reg [3:0] q = 4'b1010);
    always @(posedge clk) q <= q + 4'd1;
endmodule
```

`synth_ecp5` reports `Set init value: \q = 4'1010`, then emits four `TRELLIS_FF` cells with no `INIT`. The ECP5 global set and reset can only drive all-zero, so an all-zero initializer is satisfied implicitly and a mixed one is not implemented at all. The silence is the problem: a source-level initializer that the device ignores reads as a guarantee.

`code` now starts at `16'd0`, which is what the silicon does. The first pass sweeps from zero upwards and every later pass stays inside the valid range, so the comparison count is unchanged in meaning and the sweep claim is unaffected. This matters beyond this file: any module that initializes a register to a pattern other than all-zeroes gets zeroes on this device and no diagnostic.

## Bench state

Programmed and reloaded through `PROGRAMN` on `PB7`. The observation that closes the streaming section is whether A11 blinks, A12 stays dark, and B11 lights after about seven seconds. The recorded result is the bench report, not the build.

## Splitting stage 2, and what the multiplier costs

The two-stage split left one stage carrying two multipliers in series: `j_q` into the multiply by 781, out as `ii`, into the multiply by 21, out as `mm`, 12.18 ns in total. That is the longest path, so it sets the clock.

The fix is to notice what the two constants look like. 9363 and 781 have no shift-and-add form, so they keep a multiplier each. 21 and 28 do have one: 21 = 16 + 4 + 1 and 28 = 32 - 4. Moving the remainders into a third stage replaces two multipliers with adder trees.

| Stage | Work |
|---|---|
| 1 | `j = (q4 * 9363) >> 16` |
| 2 | `i = (j * 781) >> 14` |
| 3 | `m = j - 21 * i`, `f = offset - 28 * j` |

`f` is written as `offset - 28 * j` rather than `4 * (q4 - 7 * j) + r2`. The two are equal modulo 2^14, so the low five bits agree for every 16-bit input and not only inside the valid range. That also removes the `q4` and `r2` pipeline registers, since the offset alone carries what the later stages need.

| Metric | Two stages | Three stages |
|---|---|---|
| Fmax at a 25 MHz constraint | 77.69 MHz | 127.94 MHz |
| MULT18X18D | 4 | 2 |
| TRELLIS_FF | 43 | 58 |
| TRELLIS_COMB | 39 | 51 |
| Latency | 2 clocks | 3 clocks |
| Throughput | 1 code point per clock | 1 code point per clock |

The critical path is now stage 1: `counter` into the offset subtract, into the multiply by 9363, into `j_q`, at 7.82 ns. The multiplier's 3.93 ns of logic plus the routing around it is the floor for this shape. Registering the offset subtract as its own stage would leave about 6.5 ns, which is roughly 154 MHz, at the cost of a fourth stage and one more clock of latency. That trade is not taken here: a fifth of the clock for a fourth of the latency budget is a poor exchange at the rates this board can supply.

Simulation carries over unchanged in form and one clock longer:

```
PASS: 44689 outputs checked over 44692 clocks, 4 full sweeps of the valid space, one code point per clock
```

## The check build after the split

The self-checking build picks up the three-stage pipeline and the deeper comparator delay line.

| Metric | Value |
|---|---|
| Fmax, register to register | 58.20 MHz, PASS at 40 MHz |
| Fmax, reference path | 52.82 MHz, PASS at 40 MHz |
| TRELLIS_COMB | 176 |
| TRELLIS_FF | 154 |
| MULT18X18D | 5 |
| EHXPLLL | 1 |

The binding path is now the combinational reference, which the split did not touch: 18.9 ns, or 52.82 MHz. The pipeline alone reaches 127.94 MHz, so the check stays at 40 MHz and the margin on the reference is about 32 percent. Running the check at the pipeline's own rate would need a second clock domain and a crossing between the reference and the pipeline, which is a different design from a comparison at one clock.

Programmed and reloaded through `PROGRAMN` on `PB7` at 12:46:33. The indicators and the reading are the same as before: A11 blinks, A12 stays dark, B11 lights after 6.71 s.
