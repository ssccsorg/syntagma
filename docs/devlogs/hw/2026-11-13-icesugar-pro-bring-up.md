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
