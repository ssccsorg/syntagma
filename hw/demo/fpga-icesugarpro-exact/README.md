# fpga-icesugarpro-exact

The alphabet compared one value at a time on an iCESugar-Pro: `tagma_core`
writes one anchor per valid offset, the host writes the same set twice with two
packs exchanged between the files, and the die answers green on the file the
reference wrote and red on the file whose values moved.

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

## Why a sum is not enough

`hw/demo/fpga-icesugarpro-alphabet` gives the die the alphabet and a constant
derived from it, and the die answers green when the sum of the packs it decodes
reaches that constant. A sum answers one question and loses the rest. It cannot
say which coordinate was wrong, and two packs that trade value leave the total
untouched, so a file with two values exchanged and a file with none read the same
to it.

This package is the check that survives the exchange. The input is an anchor file
rather than a set of values, each line carrying the offset it names and the pack
the reference computed for it. The die decodes the line's offset and compares its
own pack with the pack the line carries, so the verdict is a comparison per line
and a single wrong value is the same size as eleven thousand.

## The two images

| | `anchor_pass.bit` | `anchor_swap.bit` |
|---|---|---|
| The lines | the anchors as `tagma_core` wrote them | the same lines, packs at 2 and 5 exchanged |
| The sum of the packs | `065BEA26` | `065BEA26` |
| `pin_match` | 1 | 0 |
| At stage 3 | green | red |

The two files are generated from the anchors by `hw/tools/anchor_fixtures.py`,
which refuses to write them unless the sums match, because a pair whose sums
differ would be solved by the sum image and would prove nothing about this one.
The offsets stay in order in both files, so line n names offset n and nothing
about a line's position moves.

## What the die does

The image reads one line per clock, decodes `0xAC00` plus the line's offset into
the three axes, and counts the lines whose pack disagrees with the line's own. A
line whose offset is outside the alphabet is counted separately and kept out of
the comparison, because the reference has no axes for a code point it cannot
name. `pin_match` is set only when a full pass closes with both counts zero, so
the verdict is a statement about the file as a whole.

`COUNT` comes from the file the flow is building, so the die cannot be told a
length its ROM does not have. The ROM takes 18 of the 56 DP16KD blocks the part
holds, and the design closes at 70.46 MHz against a 40 MHz clock.

The host reads the same file with `hw/tools/tagma_checksum.py`. The two
implementations share the file and nothing else: the host is Python, the die is
Verilog lowered to LUTs, and the die's decoder was not generated from the host's
code.

## What it does not claim

No rate. The die decodes one line per clock at 40 MHz and reports a verdict, and
the demonstration is that an exchange a sum cannot see is refused by a part that
was never given the host's code. The timing of the pipeline is reported by the
flow, in `hw/synth/yosys/reports/`, and not by this demonstration.

No claim about the anchors themselves. This package shows that a second
implementation reaches the same per-line answer. What the anchors mean, and why
the packing is exact, is `README.md` at the top of the repository.
