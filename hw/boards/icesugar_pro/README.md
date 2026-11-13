# iCESugar-Pro v1.3

Lattice ECP5 LFE5U-25F-6BG256C in a DDR SODIMM module, seated in its Ext-Board.
This directory owns the constraints for that board only.

## Build

```sh
make -C hw pnr BOARD=icesugar_pro              # bring-up design, RGB LED only
make -C hw pnr BOARD=icesugar_pro DESIGN=tagma_demo   # the demo
make -C hw pnr BOARD=icesugar_pro DESIGN=stream       # pipelined decoder, timing only
make -C hw pnr BOARD=icesugar_pro DESIGN=stream_check # pipelined decoder, checked on the board
```

`nextpnr-ecp5` is not packaged by Homebrew. Put OSS CAD Suite on `PATH`; `ecppack`
comes from `prjtrellis`.

## Program

The module and the Ext-Board each carry a debugger. Only the module's iCELink
reaches the ECP5.

| Debugger | USB ID | Volume | Reaches the ECP5 |
|---|---|---|---|
| module iCELink | `0x1d50:0x602b` | `iCELink` | yes |
| Ext-Board DAPLink | `0x0d28:0x0204` | `DAPLINK` | no, JTAG scans all ones |

```sh
icesprog -w hw/synth/yosys/out/tagma_demo_top.bit
```

`icesprog` is built from `tools/src` in the icesugar repository. The SRAM path
through the shipped `blink_sram.svf` fails its final status check on this board,
so flash is the programming route. To load a freshly written bitstream without
replugging, pulse `PROGRAMN` on `PB7` low then high.

## Demo wiring

Every signal group sits on one header. Wire by the ball name printed on the
Ext-Board; the column numbers below follow `doc/iCESugar-pro-pinmap.png` and are
a convenience, because the two label rows there are easy to swap.

`code[15:0]` on P3, 3.3V:

| column | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| row 1 | P7 code[0] | R6 code[1] | T4 code[3] | T3 code[4] | GND | 3V3 | T2 code[5] | GND | R1 code[6] | P1 code[7] | N1 code[8] | M1 code[9] | K1 | GND | 3V3 |
| row 2 | T6 code[2] | R5 code[10] | R4 code[11] | R3 code[12] | GND | 3V3 | R2 code[13] | 5V | P2 code[14] | N3 code[15] | M2 | L2 | G2 | GND | 3V3 |

`i[4:0]`, `m[4:0]`, `f[4:0]` on P2, 3.3V except `f[4]` at 2.5V:

| column | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| row 1 | F2 i[0] | D1 i[1] | C1 i[2] | B1 i[3] | GND | 3V3 | B3 m[2] | GND | B4 m[3] | B5 m[4] | B6 f[0] | B7 f[1] | B8 f[4] | GND | 3V3 |
| row 2 | E1 i[4] | C2 m[0] | B2 m[1] | A2 f[2] | GND | 3V3 | A3 f[3] | 5V | A4 | A5 | A6 | A7 | A8 | GND | 3V3 |

`led_valid` is on ball A12, one of the three RGB LED balls, and is active high, so validity is visible without any of the P2 wiring. The demo flow overrides the RTL's `LED_ACTIVE_LOW` to 0, which is what makes the LED light on a valid code.

The colours come from `schematic/iCESugar-Pro-v1.3.pdf` in the vendor repository, where three diodes carry the channel pins to three balls.

| Ball | Package pin | Diode | Channel |
|---|---|---|---|
| A11 | PT53A | D303, `G-` `G+` | green |
| A12 | PT53B | D301, `B-` `B+` | blue |
| B11 | PT49A | D302, `R-` `R+` | red |

The vendor README lists the same three balls as `{A11, A12, B11}`, an order that reads like red, green, blue and is not. The constraints use the schematic mapping, and the bench observation agrees with it: A12 is blue.

## Streaming indicators

`DESIGN=stream_check` needs no external wiring. Its source is an internal counter, and it compares the pipelined decoder against the combinational one on every clock from a 40 MHz PLL output. A mismatch latches.

| Signal | Ball | Meaning |
|---|---|---|
| `led_clk` | A11 | blinks at 2.4 Hz while the fast clock runs |
| `led_err` | A12 | lights and latches on any mismatch |
| `led_swept` | B11 | lights once 2^28 clocks have been compared |

The third indication is a rate reading, because the comparison saturates and never clears. 2^28 clocks is 6.71 s at 40 MHz and 10.74 s at 25 MHz, so the interval from configuration to that ball lighting separates the two clocks with a stopwatch. A pass is a blinking A11, a dark A12, and a lit B11.

`DESIGN=stream` runs the pipeline from the board oscillator and reports the register-to-register clock without driving the LEDs, so it is a timing build rather than a bench build.

Bank 1 holds the RGB LED and runs at 2.5V, which is why `B8` and `A12` are
`LVCMOS25`. Everything else is `LVCMOS33`. The `code` inputs are pulled up, so
an unwired P3 reads `0xFFFF`, which is invalid and leaves the LED dark.

## Demo verification order

Bit 14 is the only single ground that yields a valid code. Pull-ups make an
unwired header read `0xFFFF`; clearing bit 14 gives `0xBFFF`, while clearing any
other single bit leaves a value outside U+AC00..U+D7A3. Bit 15 gives `0x7FFF`,
bit 13 gives `0xDFFF`, and every lower bit gives a value still above `0xD7A3`.

| Wires to GND | `code` | Decode | LED |
|---|---|---|---|
| none | `0xFFFF` | invalid | dark |
| `code[14]`, ball P2, P3 row 2 column 9 | `0xBFFF` | i=8, m=14, f=23 | lit |
| `code[14]` and `code[12]`, ball R3, P3 row 2 column 4 | `0xAFFF` | i=1, m=15, f=15 | lit |

The first two rows need no axis LEDs, so they check the input path and the
validity output before any output wiring exists. The third straps a second bit
and moves all three axes, which is the first check that distinguishes decode
values rather than only validity.

After that, wire the 16 switches on P3 and the 15 axis LEDs on P2 as tabulated
above, and any code in the range can be dialled in. Bit 14 on P2 is also the bit
that decides the single-wire result, so a misread row between the two headers
of P3 shows up as an LED that stays dark for every switch setting.

## Verification channel

The iCELink carries four pins to four ECP5 balls, which the vendor calls JTAG2, and `icesprog` reaches them as GPIO. That is enough to read a verdict off the board over the programming cable, with no external wiring.

| ECP5 ball | iCELink pin | Host reads | Host drives |
|---|---|---|---|
| F5 | PA14 | yes | yes |
| H5 | PA13 | yes | no, the firmware keeps it |
| N4 | PA0 | yes | yes |
| J5 | PA1 | yes | yes |

`DESIGN=verify` builds `tagma_pipe_verify_top`, which accumulates a checksum over one sweep of the valid range and compares it with a constant derived from `hw/rtl/golden_anchors.hex`. The clock and the sweep counters leave the die as two divided clocks.

```sh
make -C hw pnr BOARD=icesugar_pro DESIGN=verify
icesprog -w hw/synth/yosys/out/tagma_pipe_verify_top_40mhz.bit
make -C hw board-verify ICESPROG=/path/to/icesprog
```

The first line needs OSS CAD Suite on `PATH` and the second needs `PROGRAMN` pulsed on `PB7` to reload. The third reports `pin_match` and the frequency of both dividers, and exits non-zero on a mismatch.

`VERIFY_MHZ` sets the PLL output, which is how the working rate was bracketed. The image is named after the rate it was built for.

```sh
make -C hw pnr BOARD=icesugar_pro DESIGN=verify VERIFY_MHZ=100
make -C hw board-verify ICESPROG=/path/to/icesprog BOARD_VERIFY_ARGS="--clock 100"
```

Timing closes at 125 MHz and `nextpnr-ecp5` rejects 130 MHz, so any rate above 125 will not produce a bitstream.
