#!/usr/bin/env bash
# Cut a published take: the console, the footage beside it, and the narration.
#
# The format this builds is `demo/FORMAT.md`. This script implements it and takes
# everything the format leaves to a package from the environment, because which
# two events a take is aligned on is the package's business, not the format's. A
# package supplies them through its `cut.sh`.
#
# The two anchors are the reason this is not one offset. The footage is a
# separate take of the same physical sequence, and the step between the anchors
# takes a little longer or shorter each time, so an offset that puts the first
# event exactly right leaves the second one seconds off. Two anchors give an
# offset and a ratio. `setpts` multiplies timestamps, so the ratio is console
# seconds per footage second:
#
#   out(t) = (source(t) - trim) * ratio
#   ratio  = (second - first) / (footage_b - footage_a)
#   trim   = footage_a - first / ratio
#
# Usage: demo/_shared/final.sh TAKE FOOTAGE [OUT]
#
#   ANCHOR_A      the line in the cast that marks the first anchor  (required)
#   ANCHOR_B      the line that marks the second                    (required)
#   FOOTAGE_A     where that event is in the footage, in seconds    (required)
#   FOOTAGE_B     where the second one is, in seconds               (required)
#   CAPTION_SIZE  caption point size                         (default 30)
#   CAPTION_STRIP height of the strip under the console       (default 160)
#   FONT_PATH     caption font                               (default Menlo)
#   BOARD_X       where the footage starts, across            (default 920)
#   BOARD_Y       where it starts, down                       (default 40)

set -euo pipefail
cd "$(dirname "$0")/../../.."

take=${1:?give the take directory}
board=${2:?give the footage}
out=${3:-$take/final.mp4}
anchor_a=${ANCHOR_A:?give ANCHOR_A, the line in the cast that marks the first anchor}
anchor_b=${ANCHOR_B:?give ANCHOR_B, the line that marks the second}
footage_a=${FOOTAGE_A:?give FOOTAGE_A, where the first anchor is in the footage}
footage_b=${FOOTAGE_B:?give FOOTAGE_B, where the second one is}
size=${CAPTION_SIZE:-30}
strip=${CAPTION_STRIP:-160}
font=${FONT_PATH:-/System/Library/Fonts/Menlo.ttc}
board_x=${BOARD_X:-920}
board_y=${BOARD_Y:-40}

for file in "$take/demo.mp4" "$take/demo.cast" "$take/demo.srt" "$board"; do
    [ -f "$file" ] || { echo "$file does not exist" >&2; exit 1; }
done

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

python3 - "$take" "$anchor_a" "$anchor_b" "$footage_a" "$footage_b" \
         "$work" "$font" "$size" "$strip" "$board_x" "$board_y" <<'PY'
import json, os, subprocess, sys

(take, anchor_a, anchor_b, footage_a, footage_b, work, font, size, strip,
 board_x, board_y) = sys.argv[1:12]
footage_a = float(footage_a)
footage_b = float(footage_b)
size = int(size)
strip = int(strip)

header, first, second = {}, None, None
with open(os.path.join(take, "demo.cast"), encoding="utf-8") as handle:
    for line in handle:
        line = line.strip()
        if not line:
            continue
        if line.startswith("{"):
            header = json.loads(line)
            continue
        if not line.startswith("["):
            continue
        entry = json.loads(line)
        if len(entry) < 3 or entry[1] != "o":
            continue
        when, text = float(entry[0]), entry[2]
        if first is None and anchor_a in text:
            first = when
        if second is None and anchor_b in text:
            second = when

for name, value, anchor in (("first", first, anchor_a), ("second", second, anchor_b)):
    if value is None:
        raise SystemExit("the cast never prints %r, so the footage cannot be timed" % anchor)

theme = header.get("theme", {})
bg = theme.get("bg", "#1e2030").lstrip("#")
fg = theme.get("fg", "#f2f2f2").lstrip("#")

ratio = (second - first) / (footage_b - footage_a)
trim = footage_a - first / ratio
# A film can start before the take's clock, which asks for a negative trim. The
# first frame is cloned for the difference instead, so the picture is there from
# the start and every later timestamp is where the two anchors put it.
pad = max(0.0, -trim)
trim = max(0.0, trim)

cues = []
with open(os.path.join(take, "demo.srt"), encoding="utf-8") as handle:
    body = handle.read().replace("\r\n", "\n")

def seconds(stamp):
    hours, minutes, rest = stamp.split(":")
    whole, millis = rest.split(",")
    return int(hours) * 3600 + int(minutes) * 60 + int(whole) + int(millis) / 1000

for block in body.split("\n\n"):
    rows = [row for row in block.split("\n") if row.strip()]
    if not rows:
        continue
    start, end = [part.strip() for part in rows[1].split("-->")]
    lines = rows[2:]
    png = os.path.join(work, "cue%03d.png" % len(cues))
    command = ["magick", "-background", "none", "-fill", "#" + fg,
               "-font", font, "-pointsize", str(size)]
    command += ["label:" + row for row in lines]
    command += ["-append", "PNG32:" + png]
    subprocess.run(command, check=True)
    cues.append((png, seconds(start), seconds(end)))

height = 1080 + strip
board_filter = "[1:v]"
if pad > 0:
    board_filter += "tpad=start_duration=%.3f:start_mode=clone," % pad
board_filter += ("trim=start=%.3f,setpts=(PTS-STARTPTS)*%.5f,"
                 "scale=960:540:flags=lanczos,setsar=1[board]" % (trim, ratio))

graph = [
    "[0:v]pad=1920:%d:0:0:color=0x%s[base]" % (height, bg),
    board_filter,
    "[base][board]overlay=%s:%s:eof_action=repeat[b0]" % (board_x, board_y),
]
for index, (_, start, end) in enumerate(cues):
    graph.append(
        "[b%d][%d:v]overlay=x=(W-w)/2:y=H-h-24:"
        "enable='between(t,%.3f,%.3f)'[b%d]"
        % (index, index + 2, start, end, index + 1))
graph = ";\n".join(graph) + ";\n[b%d]null[v]" % len(cues)

with open(os.path.join(work, "graph.txt"), "w") as handle:
    handle.write(graph)
with open(os.path.join(work, "count.txt"), "w") as handle:
    handle.write("%d\n" % len(cues))

print("console  first %6.2fs, second %6.2fs" % (first, second))
print("footage  first %6.2fs, second %6.2fs" % (footage_a, footage_b))
print("trim %.3fs, pad %.3fs, ratio %.5f, %d cues, frame 1920x%d"
      % (trim, pad, ratio, len(cues), height))
PY

count=$(cat "$work/count.txt")

inputs=(-i "$take/demo.mp4" -i "$board")
for index in $(seq 0 $((count - 1))); do
    inputs+=(-i "$work/$(printf 'cue%03d.png' "$index")")
done

# The track is the same narration the captions carry, built by voice.sh on the
# video's timeline, so a take that has one is published with sound.
if [ -f "$take/voice.wav" ]; then
    inputs+=(-i "$take/voice.wav")
    audio=(-map "$((count + 2)):a" -c:a aac -b:a 128k)
else
    audio=(-an)
fi

ffmpeg -v error -y "${inputs[@]}" -filter_complex "$(cat "$work/graph.txt")" \
    -map "[v]" "${audio[@]}" \
    -c:v libx264 -preset medium -crf 20 -pix_fmt yuv420p \
    -movflags +faststart -r 30 "$out"

echo
echo "$out"
ffprobe -v error -show_entries stream=width,height -of csv=p=0 "$out"
ffprobe -v error -show_entries format=duration -of default=nw=1 "$out"
ls -l "$out" | awk '{print $5 " bytes"}'
