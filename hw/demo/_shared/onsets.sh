#!/usr/bin/env bash
# Read the LED out of a film, so a cut's anchors are measured rather than guessed.
#
# The crop is where the light sits in the frame, given as x:y:w:h. Find it once
# with a still:
#
#   ffmpeg -ss 30 -i film.mov -frames:v 1 /tmp/frame.png
#
# and read the LED's box off that still. A 48-pixel square around it is enough.
#
# Usage: onsets.sh FILM X:Y:W:H [START END [STEP]]
#
#   onsets.sh film.mov 1024:344:48:48
#   onsets.sh film.mov 1024:344:48:48 60 175 0.25
#
# Each line is the mean colour of the crop at that second, and a marker on the
# lines where the colour changed. The label is a guess from that mean, and a
# blinking stage alternates between its colour and dark, so read the steady
# transitions. The two the cut needs, red and green, are both steady.

set -euo pipefail
film=${1:?give the film}
crop=${2:?give the box of the LED as x:y:w:h}
start=${3:-0}
end=${4:-}
step=${5:-1}

[ -f "$film" ] || { echo "$film does not exist" >&2; exit 1; }
if [ -z "$end" ]; then
    end=$(ffprobe -v error -show_entries format=duration -of csv=p=0 "$film")
fi

python3 - "$film" "$crop" "$start" "$end" "$step" <<'PY'
import subprocess, sys

film, crop = sys.argv[1], sys.argv[2]
start, end, step = float(sys.argv[3]), float(sys.argv[4]), float(sys.argv[5])

# The filter takes width and height first, and the box is given the other way
# round, because that is the order a still is read in.
x, y, w, h = [int(value) for value in crop.split(":")]
crop = "%d:%d:%d:%d" % (w, h, x, y)

def sample(at):
    out = subprocess.run(
        ["ffmpeg", "-v", "error", "-ss", "%.2f" % at, "-i", film, "-frames:v", "1",
         "-vf", "crop=%s,scale=1:1" % crop, "-f", "rawvideo", "-pix_fmt", "rgb24", "-"],
        capture_output=True).stdout
    return tuple(out[:3]) if len(out) >= 3 else None

def label(r, g, b):
    # The bloom drives all three channels high, so the colour is read from the
    # differences rather than from the levels.
    if max(r, g, b) < 90:
        return "off"
    if r - g >= 30 and r >= 150:
        return "red"
    if g - r >= 30 and g - b >= 20:
        return "green"
    if g - b >= 25 and r >= 150:
        return "yellow"
    if b - g >= 30:
        return "blue"
    if b - g >= 0 and g >= 100:
        return "cyan"
    return "dim"

at = start
previous = None
while at <= end:
    rgb = sample(at)
    if rgb is None:
        break
    colour = label(*rgb)
    mark = "  <-" if colour != previous else ""
    print("%7.2f  %3d %3d %3d  %-6s%s" % (at, rgb[0], rgb[1], rgb[2], colour, mark))
    previous = colour
    at += step
PY
