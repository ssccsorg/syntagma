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
"""Render a Sky130 GDS layout to a filtered PNG, with an optional SVG.

The source is the OpenROAD flow's final GDS. The image keeps the metal stack
and leaves the FEOL and the boundary layers out. By default the frame is the
placed logic, read from the sibling DEF, so the fill rows do not set the crop.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

import gdstk
import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt
from matplotlib.collections import PolyCollection

# Sky130 layer numbers in the ORFS flow. The FEOL and the boundary layers are not part of the image.
# The colour set is for screen; the mono set is for print, where the stack has to read on white.
PALETTES = {
    "color": {
        67: "#3b4b5c",  # li1
        68: "#2f6f9f",  # met1
        69: "#2f9f8f",  # met2
        70: "#c9a227",  # met3
        71: "#b05fb0",  # met4
        72: "#e6e1d6",  # met5
    },
    "mono": {
        67: "#a8a8a8",  # li1, the densest texture and the palest
        68: "#8a8a8a",  # met1
        69: "#6f6f6f",  # met2
        70: "#565656",  # met3
        71: "#363636",  # met4
        72: "#111111",  # met5, the power grid and the darkest
    },
}
BACKGROUND = "#11161c"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--gds", required=True, type=Path)
    parser.add_argument("--def", dest="def_path", type=Path, help="DEF beside the GDS by default")
    parser.add_argument("--out", type=Path, default=Path(__file__).resolve().parent / "out")
    parser.add_argument("--top", help="top cell name; the first top-level cell by default")
    parser.add_argument(
        "--layers",
        default=",".join(str(number) for number in PALETTES["color"]),
        help="comma-separated GDS layer numbers",
    )
    parser.add_argument(
        "--palette",
        choices=sorted(PALETTES),
        default="color",
        help="colour for screen, mono for print on white",
    )
    parser.add_argument(
        "--crop",
        default="auto",
        help="auto (placed logic), die, or x0,y0,x1,y1 in microns",
    )
    parser.add_argument("--margin", type=float, default=10.0, help="crop margin in microns")
    parser.add_argument("--size", type=int, default=2400, help="long edge in pixels")
    parser.add_argument("--dpi", type=int, default=300)
    parser.add_argument("--background", default=None)
    parser.add_argument("--name", default="silicon")
    parser.add_argument("--merge", action="store_true", help="union overlapping shapes per layer")
    parser.add_argument("--frame", action="store_true", help="draw the die boundary")
    parser.add_argument("--svg", action="store_true", help="also write an SVG")
    return parser.parse_args()


def parse_def_crop(path: Path, margin: float) -> tuple[float, float, float, float] | None:
    """The placed logic cells, with fillers left out, in microns."""
    if not path.is_file():
        return None
    text = path.read_text(encoding="utf-8", errors="replace")
    units_match = re.search(r"UNITS DISTANCE MICRONS\s+(\d+)", text)
    if not units_match or "COMPONENTS" not in text:
        return None
    units = int(units_match.group(1))
    body = text.split("COMPONENTS", 1)[1].split("END COMPONENTS", 1)[0]
    xs: list[float] = []
    ys: list[float] = []
    for line in body.splitlines():
        placed = re.match(
            r"\s*-\s+(\S+)\s+(\S+).*?PLACED\s+\(\s*(-?\d+)\s+(-?\d+)\s*\)",
            line,
        )
        if not placed:
            continue
        name, cell, x, y = placed.groups()
        if name.startswith("FILLER_") or "__fill" in cell or "__decap" in cell:
            continue
        xs.append(int(x) / units)
        ys.append(int(y) / units)
    if not xs:
        return None
    return (min(xs) - margin, min(ys) - margin, max(xs) + margin, max(ys) + margin)


def main() -> int:
    args = parse_args()
    if not args.gds.is_file():
        raise SystemExit(f"GDS not found: {args.gds}")

    library = gdstk.read_gds(str(args.gds))
    tops = library.top_level()
    if not tops:
        raise SystemExit(f"no top-level cell in {args.gds}")
    if args.top:
        matches = [cell for cell in tops if cell.name == args.top]
        if not matches:
            raise SystemExit(f"top cell {args.top!r} not in {[cell.name for cell in tops]}")
        cell = matches[0]
    else:
        cell = tops[0]

    layers = {int(value) for value in args.layers.split(",") if value.strip()}
    palette = PALETTES[args.palette]
    background = args.background or ("white" if args.palette == "mono" else BACKGROUND)
    unknown = layers - set(palette)
    if unknown:
        raise SystemExit(f"layer(s) without a color: {sorted(unknown)}")

    shapes: dict[int, list] = {}
    for polygon in cell.get_polygons():
        if polygon.layer in layers:
            shapes.setdefault(polygon.layer, []).append(polygon)
    if not shapes:
        raise SystemExit(f"no polygons on layers {sorted(layers)} in {cell.name}")

    # Overlapping shapes on one layer draw the same, so a union keeps the image and cuts the vertex count.
    polygons: dict[int, list] = {}
    for layer, layer_shapes in shapes.items():
        if args.merge:
            layer_shapes = gdstk.boolean(layer_shapes, [], "or", precision=1e-4, layer=layer)
        polygons[layer] = [polygon.points for polygon in layer_shapes]

    bounds = cell.bounding_box()
    if bounds is None:
        raise SystemExit(f"empty bounding box in {cell.name}")
    (die_x0, die_y0), (die_x1, die_y1) = bounds

    def_path = args.def_path or args.gds.with_suffix(".def")
    if args.crop == "die":
        x0, y0, x1, y1 = die_x0, die_y0, die_x1, die_y1
    elif args.crop == "auto":
        crop = parse_def_crop(def_path, args.margin)
        x0, y0, x1, y1 = crop or (die_x0, die_y0, die_x1, die_y1)
    else:
        values = [float(value) for value in args.crop.split(",")]
        if len(values) != 4:
            raise SystemExit("--crop takes auto, die, or x0,y0,x1,y1")
        x0, y0, x1, y1 = values

    x0 = max(x0, die_x0)
    y0 = max(y0, die_y0)
    x1 = min(x1, die_x1)
    y1 = min(y1, die_y1)
    if x1 <= x0 or y1 <= y0:
        x0, y0, x1, y1 = die_x0, die_y0, die_x1, die_y1

    width = x1 - x0
    height = y1 - y0
    long_edge = args.size / args.dpi
    if width >= height:
        fig_width = long_edge
        fig_height = long_edge * height / width
    else:
        fig_height = long_edge
        fig_width = long_edge * width / height

    fig, ax = plt.subplots(figsize=(fig_width, fig_height), facecolor=background)
    ax.set_facecolor(background)
    for layer in sorted(polygons):
        ax.add_collection(
            PolyCollection(
                polygons[layer],
                facecolors=palette[layer],
                edgecolors="none",
                antialiased=True,
            )
        )
    ax.set_xlim(x0, x1)
    ax.set_ylim(y0, y1)
    if args.frame:
        # The axes fill the figure, so the border is inset to keep its stroke on the canvas.
        inset = (x1 - x0) * 0.004
        ax.add_patch(
            plt.Rectangle(
                (die_x0 + inset, die_y0 + inset),
                die_x1 - die_x0 - 2 * inset,
                die_y1 - die_y0 - 2 * inset,
                fill=False,
                edgecolor=palette[max(layers)] if args.palette == "mono" else "#000000",
                linewidth=1.0,
            )
        )
    ax.set_aspect("equal")
    ax.axis("off")
    fig.subplots_adjust(left=0, right=1, bottom=0, top=1)

    args.out.mkdir(parents=True, exist_ok=True)
    png = args.out / f"{args.name}.png"
    fig.savefig(png, dpi=args.dpi, facecolor=background)
    if args.svg:
        fig.savefig(args.out / f"{args.name}.svg", format="svg", facecolor=background)
    plt.close(fig)

    print(f"top: {cell.name}")
    print(f"crop: {x0:.2f}, {y0:.2f}, {x1:.2f}, {y1:.2f} um")
    for layer in sorted(polygons):
        print(f"  layer {layer}: {len(polygons[layer])} polygons")
    print(f"wrote {png}")
    if args.svg:
        print(f"wrote {args.out / f'{args.name}.svg'}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
