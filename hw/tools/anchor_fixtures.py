#!/usr/bin/env python3
# SPDX-License-Identifier: CERN-OHL-S-2.0
# Copyright (C) 2026 Taeho Lee
#
# This source describes Open Hardware and is licensed under the CERN-OHL-P v2.
# You may redistribute and modify this documentation and make products using
# it under the terms of the CERN-OHL-P v2 (https://cern.ch/cern-ohl). This
# documentation is distributed WITHOUT ANY EXPRESS OR IMPLIED WARRANTY,
# INCLUDING OF MERCHANTABILITY, SATISFACTORY QUALITY AND FITNESS FOR A
# PARTICULAR PURPOSE. Please see the CERN-OHL-P v2 for applicable conditions.
"""Write the fixture pair the per-value anchor comparison is gated on.

The pass fixture is the anchor file as tagma_core wrote it. The swap fixture
carries the same lines with two packs exchanged and the offsets untouched, so
the multiset of packs is the same and a sum over them cannot separate the two
files. The decoder's answer for each offset is unchanged, so the swap differs
from the pass fixture at exactly the lines it touched.

The offsets stay in order, which is what tagma_checksum.py requires. The top does
not read a line by its position: each line carries the offset it names, so the
order is a property of the file rather than of the comparison.

Usage:
    anchor_fixtures.py [--count N] [--swap I,J] OUT_DIR

Exit status is nonzero when the two fixtures do not carry the same sum, because
the negative control is the point of writing two files.
"""

import argparse
from pathlib import Path

PACK_MASK = 0x7FFF
OFFSET_SHIFT = 15
SUM_MASK = 0xFFFFFFFF

PASS_NAME = "anchor_fixture_pass.hex"
SWAP_NAME = "anchor_fixture_swap.hex"


def read_anchors(path: Path) -> list[int]:
    lines: list[int] = []
    for index, text in enumerate(path.read_text().splitlines()):
        text = text.strip()
        if not text:
            continue
        value = int(text, 16)
        if value >> OFFSET_SHIFT != index:
            raise SystemExit(f"{path}:{index + 1}: offset does not match the line index")
        lines.append(value)
    if not lines:
        raise SystemExit(f"{path}: no anchors")
    return lines


def pack_sum(lines: list[int]) -> int:
    total = 0
    for value in lines:
        total = (total + (value & PACK_MASK)) & SUM_MASK
    return total


def exchange_packs(lines: list[int], first: int, second: int) -> list[int]:
    exchanged = list(lines)
    first_pack = lines[first] & PACK_MASK
    second_pack = lines[second] & PACK_MASK
    if first_pack == second_pack:
        raise SystemExit(f"lines {first} and {second} carry the same pack, so nothing moves")
    exchanged[first] = (lines[first] & ~PACK_MASK) | second_pack
    exchanged[second] = (lines[second] & ~PACK_MASK) | first_pack
    return exchanged


def write(lines: list[int], path: Path) -> None:
    path.write_text("".join(f"{value:08X}\n" for value in lines))


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--anchors",
        default="rtl/golden_anchors.hex",
        help="the file tagma_core wrote, relative to hw/",
    )
    parser.add_argument(
        "--count",
        type=int,
        default=0,
        help="lines to take from the front; 0 takes the whole file",
    )
    parser.add_argument(
        "--swap",
        default="2,5",
        help="the two lines whose packs are exchanged",
    )
    parser.add_argument("out_dir", help="where to write the two fixtures")
    args = parser.parse_args()

    anchors = read_anchors(Path(args.anchors))
    if args.count < 0 or args.count > len(anchors):
        raise SystemExit(f"--count {args.count} is outside 0..{len(anchors)}")
    taken = anchors[: args.count] if args.count else anchors

    first, second = (int(part) for part in args.swap.split(","))
    for index in (first, second):
        if not 0 <= index < len(taken):
            raise SystemExit(f"--swap {args.swap} is outside the {len(taken)} lines taken")

    swapped = exchange_packs(taken, first, second)

    pass_sum = pack_sum(taken)
    swap_sum = pack_sum(swapped)
    if pass_sum != swap_sum:
        raise SystemExit(
            f"the swap moved the sum: {pass_sum:08X} became {swap_sum:08X}, "
            "so the pair is not a negative control for a checksum"
        )

    out_dir = Path(args.out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    write(taken, out_dir / PASS_NAME)
    write(swapped, out_dir / SWAP_NAME)

    print(f"{PASS_NAME}: {len(taken)} lines, sum {pass_sum:08X}")
    print(
        f"{SWAP_NAME}: {len(taken)} lines, sum {swap_sum:08X}, "
        f"packs at {first} and {second} exchanged"
    )


if __name__ == "__main__":
    main()
