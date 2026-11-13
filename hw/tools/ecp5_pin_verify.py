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
"""Read the on-device verification pins through the iCELink debugger.

The device drives three ECP5 balls that the iCELink reaches as GPIOs, so the
verdict and two divided clocks are readable over the programming cable with no
external wiring. Each icesprog invocation is one USB round trip, about 16 ms
here, which sets the resolution of every measurement below.

pin_match  F5  PA14  the sweep checksum equalled the expected constant
pin_beat   H5  PA13  heart[24], a rising edge every 2^25 clocks
pin_sweep  N4  PA0   sweeps[10], a rising edge every 2048 sweeps

The beat edges give the clock directly. The sweep edges give the sweep rate,
which fixes the clock a second way because a sweep is 11,172 clocks.

Usage:
    ecp5_pin_verify.py [--icesprog PATH] [--seconds 4] [--clock 40]
"""

import argparse
import statistics
import subprocess
import sys
import time

PINS = {
    "match": "PA14",
    "beat": "PA13",
    "sweep": "PA0",
}

BEAT_EDGE_CLOCKS = 1 << 25
SWEEP_CLOCKS = 11172 * 2048


def icesprog(binary: str, *args: str) -> str:
    out = subprocess.run(
        [binary, *args], capture_output=True, text=True, check=False
    )
    if out.returncode != 0:
        raise SystemExit(f"icesprog {' '.join(args)} failed: {out.stderr.strip()}")
    return out.stdout


def set_input(binary: str, pin: str) -> None:
    icesprog(binary, "-g", pin, "-m", "in")


def read_pin(binary: str, pin: str) -> int:
    text = icesprog(binary, "-g", pin, "-r")
    for line in text.splitlines():
        if "return" in line:
            return int(line.rsplit("return", 1)[1].strip())
    raise SystemExit(f"unexpected icesprog output for {pin}: {text!r}")


def sample(binary: str, pin: str, seconds: float) -> tuple[list[float], list[int]]:
    times: list[float] = []
    values: list[int] = []
    end = time.monotonic() + seconds
    while time.monotonic() < end:
        values.append(read_pin(binary, pin))
        times.append(time.monotonic())
    return times, values


def rising_edges(times: list[float], values: list[int]) -> list[float]:
    edges: list[float] = []
    for index in range(1, len(values)):
        if values[index] == 1 and values[index - 1] == 0:
            # The edge happened between the two samples; take the midpoint.
            edges.append((times[index - 1] + times[index]) / 2.0)
    return edges


def period_seconds(edges: list[float]) -> float | None:
    """Least-squares slope of edge index against time, inverted."""
    if len(edges) < 3:
        return None
    n = len(edges)
    ks = list(range(n))
    mean_k = statistics.fmean(ks)
    mean_t = statistics.fmean(edges)
    num = sum((k - mean_k) * (t - mean_t) for k, t in zip(ks, edges))
    den = sum((k - mean_k) ** 2 for k in ks)
    if den == 0:
        return None
    return num / den


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--icesprog", default="icesprog")
    parser.add_argument("--seconds", type=float, default=6.0)
    parser.add_argument("--clock", type=float, default=40.0, help="expected MHz")
    parser.add_argument("--tolerance", type=float, default=0.05)
    args = parser.parse_args()

    for pin in PINS.values():
        set_input(args.icesprog, pin)

    match = read_pin(args.icesprog, PINS["match"])

    beat_times, beat_values = sample(args.icesprog, PINS["beat"], args.seconds)
    beat_edges = rising_edges(beat_times, beat_values)
    beat_period = period_seconds(beat_edges)

    sweep_times, sweep_values = sample(args.icesprog, PINS["sweep"], args.seconds)
    sweep_edges = rising_edges(sweep_times, sweep_values)
    sweep_period = period_seconds(sweep_edges)

    print(f"pin_match = {match}")
    print(f"beat samples {len(beat_values)}, edges {len(beat_edges)}")
    print(f"sweep samples {len(sweep_values)}, edges {len(sweep_edges)}")

    measured = None
    if beat_period:
        beat_hz = 1.0 / beat_period
        measured = BEAT_EDGE_CLOCKS * beat_hz / 1e6
        print(f"beat period {beat_period:.4f} s -> {measured:.2f} MHz")

    sweep_clock = None
    if sweep_period:
        sweep_clock = SWEEP_CLOCKS / sweep_period / 1e6
        print(f"sweep period {sweep_period:.4f} s -> {sweep_clock:.2f} MHz")

    ok = match == 1
    if measured is None:
        print("VERDICT inconclusive: no beat edges, the design may not be running")
        return 2
    if abs(measured - args.clock) > args.clock * args.tolerance:
        print(f"VERDICT clock off: {measured:.2f} MHz against {args.clock:.2f} MHz")
        ok = False
    if sweep_clock is not None and abs(sweep_clock - args.clock) > args.clock * args.tolerance:
        print(f"VERDICT sweep clock off: {sweep_clock:.2f} MHz")
        ok = False
    if match != 1:
        print("VERDICT checksum mismatch")
    print("VERDICT pass" if ok else "VERDICT fail")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
