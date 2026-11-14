#!/usr/bin/env python3
# SPDX-License-Identifier: CERN-OHL-P-2.0
# Copyright (C) 2026 Taeho Lee
#
# This source describes Open Hardware and is licensed under the CERN-OHL-P v2.
# You may redistribute and modify this documentation and make products using
# it under the terms of the CERN-OHL-P v2 (https://cern.ch/cern-ohl). This
# documentation is distributed WITHOUT ANY EXPRESS OR IMPLIED WARRANTY,
# INCLUDING OF MERCHANTABILITY, SATISFACTORY QUALITY AND FITNESS FOR A
# PARTICULAR PURPOSE. Please see the CERN-OHL-P v2 for applicable conditions.
"""Print the checksum of the golden anchors that the device must reproduce.

Each anchor line packs offset[28:15] initial[14:10] medial[9:5] final[4:0], and
the device accumulates ((i << 10) | (m << 5) | f) over one sweep of the valid
range. Summing the same packing here yields the constant the verification top
compares against.

Usage:
    tagma_checksum.py [path/to/golden_anchors.hex]
"""

import sys
from pathlib import Path

MASK = 0xFFFFFFFF


def checksum(path: Path) -> tuple[int, int]:
    total = 0
    count = 0
    for index, line in enumerate(path.read_text().splitlines()):
        line = line.strip()
        if not line:
            continue
        packed = int(line, 16)
        if packed >> 15 != index:
            raise SystemExit(f"{path}:{index + 1}: offset does not match the line index")
        initial = (packed >> 10) & 0x1F
        medial = (packed >> 5) & 0x1F
        final = packed & 0x1F
        total = (total + (initial << 10) + (medial << 5) + final) & MASK
        count += 1
    return total, count


def main() -> None:
    path = Path(sys.argv[1]) if len(sys.argv) > 1 else Path("hw/rtl/golden_anchors.hex")
    total, count = checksum(path)
    print(f"{total:08X}")
    print(f"# {count} anchors from {path}", file=sys.stderr)


if __name__ == "__main__":
    main()
