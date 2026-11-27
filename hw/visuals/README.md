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

The target writes three images to `visuals/out/`: `silicon.png`, the die at its full frame; `silicon-detail.png`, a close crop of the dense centre; and `silicon-routing.png`, the die with the power straps withheld. Add `--svg` to a direct call for a vector copy.

Or directly:

```sh
uv run --python 3.11 --with gdstk --with matplotlib python visuals/render_gds.py \
  --gds openroad/results/tagma_demo/base/6_final.gds \
  --out visuals/out
```

`visuals/out/` is gitignored.

## What the image shows

The renderer keeps the metal stack: `li1` and `met1` through `met5`, Sky130 layers 67 to 72. The FEOL and the boundary layers are left out. `LAYERS` in `render_gds.py` states the set and the palette; `--layers` narrows the set.

By default the frame is the placed logic cells, read from the sibling `6_final.def` with the fillers excluded and a 10 um margin. `--crop die` keeps the full die; `--crop x0,y0,x1,y1` sets an explicit window.

The image is a 2D layout view. gdstk does not render 3D. A stylized 3D extrusion is a different tool and should be labeled as a visualization rather than a layout view.
