<!-- SPDX-License-Identifier: CERN-OHL-S-2.0 -->
<!-- Copyright (C) 2026 Taeho Lee -->

# GDS renders

The renderer turns the Sky130 layout the OpenROAD flow produces into a PNG, with an optional SVG. The renderer belongs to this directory; the GDS and the images are build outputs.

## Source

`openroad/run.sh` runs the ORFS Sky130 flow and copies its results to `openroad/results/tagma_demo/base/6_final.gds`. `openroad/results/` is gitignored. A CI run of the `hw` job carries the same directory as the `sky130-reports` artifact.

## Render

From `hw/`:

```sh
make visuals
```

The target writes three images to `visuals/out/`: `silicon-detail.png`, a close crop of the dense centre; `silicon-routing.png`, the whole die with the power straps withheld; and `silicon-tile.png`, a 16:9 still for the corpus hub's grid. The die at its full frame is the hub's 3D view, so it is not written as a separate still. Add `--svg` to a direct call for a vector copy.

`make visuals-stack` writes one transparent plate per metal layer to `visuals/out/stack/`, the six files the hub's 3D view stacks. The plates are the whole square die, so they read as the die from above and separate into the metal levels when turned.

`make visuals-print` writes the two figures the SSCCS documentation embeds: `gds-die.svg` and `gds-routing.svg`, the whole die in monochrome with the power straps and without them. They are vector, on white, and union per layer so each stays under a megabyte. Colour is the default for every other target; mono is only for print.

Or directly:

```sh
uv run --python 3.11 --with gdstk --with matplotlib python visuals/render_gds.py \
  --gds openroad/results/tagma_demo/base/6_final.gds \
  --out visuals/out
```

`visuals/out/` is gitignored.

## What the image shows

The renderer keeps the metal stack: `li1` and `met1` through `met5`, Sky130 layers 67 to 72. The FEOL and the boundary layers are left out. `PALETTES` in `render_gds.py` states the two palettes and `--layers` narrows the set.

`--palette color` draws for a dark screen and is the default; `--palette mono` runs pale to dark by layer on white, so the power grid (`met5`) is the darkest element in print. `--merge` unions the overlapping shapes of each layer before drawing, which keeps the image and cuts the vertex count by about four times on the dense layers.

By default the frame is the placed logic cells, read from the sibling `6_final.def` with the fillers excluded and a 10 um margin. `--crop die` keeps the full die; `--crop x0,y0,x1,y1` sets an explicit window.

The image is a 2D layout view. gdstk does not render 3D. A stylized 3D extrusion is a different tool and should be labeled as a visualization rather than a layout view.
