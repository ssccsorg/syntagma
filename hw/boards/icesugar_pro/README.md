# iCESugar-Pro v1.3

Lattice ECP5 LFE5U-25F-6BG256C in a DDR SODIMM module, seated in its Ext-Board.
This directory owns the constraints for that board only.

## Build

```sh
make -C hw pnr BOARD=icesugar_pro              # bring-up design, RGB LED only
make -C hw pnr BOARD=icesugar_pro DESIGN=tagma_demo   # the demo
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

`led_valid` is the module's RGB LED green channel on A12, so validity is visible
without any of the P2 wiring.

Bank 1 holds the RGB LED and runs at 2.5V, which is why `B8` and `A12` are
`LVCMOS25`. Everything else is `LVCMOS33`. `code` inputs are left unpulled, so
an unwired P3 floats and the validity LED is indeterminate until switches drive
the pins.
