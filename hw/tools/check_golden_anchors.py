#!/usr/bin/env python3
"""Consistency gate for the Tagma golden anchor file.

Verifies that hw/rtl/golden_anchors.hex is well-formed and that every
entry satisfies the Tagma decomposition contract. Each line is one packed
29-bit value: offset[28:15] initial[14:10] medial[9:5] final[4:0].

    offset in [0, 11171]
    i = offset / 588
    m = (offset % 588) / 28
    f = offset % 28

The gate protects the generated golden file against stale or corrupted
regeneration. The RTL cross-check itself runs in the Verilator golden
testbench (make sim-golden).

Exit code 0 on success, 1 on any violation.
"""

import sys
from pathlib import Path

N_VALID = 11172
STRIDE_INIT = 588
STRIDE_MED = 28
N_INIT = 19
N_MED = 21
N_FIN = 28


def main() -> int:
    path = Path(__file__).resolve().parents[1] / "rtl" / "golden_anchors.hex"
    errors = 0
    lines = 0
    with path.open() as fh:
        for lineno, line in enumerate(fh):
            fields = line.split()
            if len(fields) != 1:
                print(f"{path}:{lineno + 1}: expected 1 packed value, got {len(fields)}")
                errors += 1
                continue
            try:
                value = int(fields[0], 16)
            except ValueError:
                print(f"{path}:{lineno + 1}: non-hex field: {line.strip()}")
                errors += 1
                continue
            offset = value >> 15
            i = (value >> 10) & 0x1F
            m = (value >> 5) & 0x1F
            f = value & 0x1F
            if offset != lines:
                print(f"{path}:{lineno + 1}: offset 0x{offset:X} != line index 0x{lines:X}")
                errors += 1
            if offset >= N_VALID:
                print(f"{path}:{lineno + 1}: offset {offset} out of range")
                errors += 1
            if i != offset // STRIDE_INIT:
                print(f"{path}:{lineno + 1}: initial {i} != {offset // STRIDE_INIT}")
                errors += 1
            if m != (offset % STRIDE_INIT) // STRIDE_MED:
                print(f"{path}:{lineno + 1}: medial {m} != {(offset % STRIDE_INIT) // STRIDE_MED}")
                errors += 1
            if f != offset % STRIDE_MED:
                print(f"{path}:{lineno + 1}: final {f} != {offset % STRIDE_MED}")
                errors += 1
            if i >= N_INIT or m >= N_MED or f >= N_FIN:
                print(f"{path}:{lineno + 1}: axis out of bounds i={i} m={m} f={f}")
                errors += 1
            lines += 1

    if lines != N_VALID:
        print(f"{path}: expected {N_VALID} lines, got {lines}")
        errors += 1

    if errors == 0:
        print(f"golden anchors OK: {lines} entries")
        return 0
    print(f"golden anchors FAILED: {errors} violations")
    return 1


if __name__ == "__main__":
    sys.exit(main())
