# fpga-icesugarpro-alphabet

The alphabet on an iCESugar-Pro: `tagma_core` writes one anchor per valid offset,
the host sums the packs, and the die decodes every value to the same number
without sharing a line of code with the reference.

The format every package shares is `hw/demo/FORMAT.md`. What this board adds to
it, the two anchors and the status vocabulary, is in the `FORMAT.md` beside this
file, which inherits the shared one the way a nested README does. The line that
records a take is `demo.sh`, and the line that cuts it is `cut.sh`.

## What it needs

| | |
|---|---|
| The board | an iCESugar-Pro v1.3 on USB |
| `icesprog` | `ICESPROG=/path/to/icesprog` if it is not on `PATH` |
| The toolchain | `nextpnr-ecp5`, `ecppll`, `ecppack`, `yosys`, which OSS CAD Suite carries |
| The flow | `hw/Makefile` and `hw/synth/yosys/run_pnr.sh`, which are this repository |

Only the first run needs the toolchain, because `images.sh` builds the two images
once and `--no-images` reuses them.

## The two images

Both are built by the flow from the anchors, so the die is checked against what
`tagma_core` wrote rather than against itself.

| | `alphabet_pass.bit` | `alphabet_fail.bit` |
|---|---|---|
| The values | every offset of the alphabet, in order | the same, one value moved to 11,172 |
| `pin_match` | 1 | 0 |
| At stage 3 | green | red |

The values are the offsets the anchors name, so the input to the image is the
alphabet rather than a sample of something else. A value of 11,172 is outside it:
the format holds 11,172 offsets, numbered from zero, so the overflow is the first
value the die must refuse.

## What the die does

The image reads one value per clock, adds the code point base, and decodes it into
the three axes. It accumulates the packed axes and compares the total with a
constant the flow derived from `hw/rtl/golden_anchors.hex`, the file `tagma_core`
writes. A value at or above the alphabet is counted as invalid and kept out of the
sum, so one changed value both moves the total and raises the count. Either one
clears `pin_match`.

The host sums the same packs from the same file with `hw/tools/tagma_checksum.py`.
The two implementations share the file and nothing else: the host is Rust and
Python, the die is Verilog lowered to LUTs, and the die's decoder was not generated
from the host's code.

## What it does not claim

No rate. The die decodes one value per clock at 40 MHz and reports a verdict, and
the demonstration is that the alphabet is exact enough for a second implementation
to reach the same number. The timing of the pipeline is reported by the flow, in
`hw/synth/yosys/reports/`, and not by this demonstration.

Nothing here stores anything. The part decodes the alphabet and says whether it
adds up; it holds no record, runs no engine, and is handed values rather than a
medium.
