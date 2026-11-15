#!/usr/bin/env bash
# Render a cast to the frame the phone footage uses, so the two can be stacked.
#
# The frame is 1920x1080: the terminal renders at 1915x1075 with the default
# font, and the last five pixels of width and height are the terminal's own
# background, so nothing is stretched. Two of them stack to 1920x2160, and the
# middle 1920 rows are the square.
#
# agg writes a GIF and nothing else, so the video is one encode away from it.
# The encode resamples at 30 frames a second rather than copying the GIF's
# per-frame delays: a GIF carries its pauses as delays a player may clamp, and
# copying them is how a 234 second run came out short. Resampling turns each
# pause into the frames that fill it, so the output length follows the cast.
#
# Usage: demo/_shared/render.sh CAST [OUT]

set -euo pipefail
cd "$(dirname "$0")/../../.."

cast=${1:?give the cast to render}
out=${2:-${cast%.cast}.mp4}

font=${FONT:-24}
cols=${COLS:-131}
rows=${ROWS:-31}
idle=${IDLE:-1000}
fps=${FPS:-30}

command -v agg >/dev/null 2>&1 || { echo "agg is not installed" >&2; exit 1; }
command -v ffmpeg >/dev/null 2>&1 || { echo "ffmpeg is not installed" >&2; exit 1; }
[ -f "$cast" ] || { echo "$cast does not exist" >&2; exit 1; }

# The strip added to reach 1080p is the terminal's own background, taken from
# the cast so it matches whatever theme recorded it.
bg=$(python3 - "$cast" <<'PY'
import json, sys
with open(sys.argv[1], encoding="utf-8") as handle:
    header = json.loads(handle.readline())
print(header.get("theme", {}).get("bg", "#1e2030").lstrip("#"))
PY
)

gif="${out%.mp4}.gif"
agg --font-size "$font" --cols "$cols" --rows "$rows" \
    --idle-time-limit "$idle" "$cast" "$gif"

ffmpeg -v error -y -i "$gif" \
    -vf "fps=$fps,pad=1920:1080:0:0:color=0x$bg" \
    -c:v libx264 -pix_fmt yuv420p -crf 20 -movflags +faststart \
    "$out"

echo "$out"
ffprobe -v error -select_streams v:0 -show_entries stream=width,height -of csv=p=0 "$out"
ffprobe -v error -show_entries format=duration -of default=nw=1 "$out"
