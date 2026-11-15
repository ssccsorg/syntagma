#!/usr/bin/env bash
# Record one demo package.
#
#   hw/demo/_shared/run.sh hw/demo/fpga-icesugarpro-alphabet
#   hw/demo/_shared/run.sh hw/demo/fpga-icesugarpro-alphabet --preflight
#   hw/demo/_shared/run.sh hw/demo/fpga-icesugarpro-alphabet --no-images
#
# A package is a directory under hw/demo/ named for the surface it demonstrates:
# the platform, the board, and the topic. It owns the script that is recorded,
# the images built for its run, the markers a take has to carry, its takes, and
# what its cut adds to `../FORMAT.md`. The recorder, the render, the narration
# track, the cut, and the check are shared, because they work on a take rather
# than on a story.
#
# Each package carries its own `demo.sh`, so the line a presenter types is
# `hw/demo/<package>/demo.sh`.
#
# What it does, in order: checks the board and the tools, makes sure the images
# exist, puts the part in the state the take opens on, records the run at the
# frame the video uses, renders that cast to 16:9, builds the narration track and
# its captions at the times the run spoke, checks the take, and prints where the
# files are.
#
# Needs: the board on USB, icesprog, asciinema, agg, and for the first run the
# FPGA toolchain that OSS CAD Suite carries.

set -euo pipefail
cd "$(dirname "$0")/../../.."

icesprog=${ICESPROG:-icesprog}
styles=131x31
# One narration setting for the run and for the track built from it.
tts=${TTS:-voice}
tts_voice=${TTS_VOICE:-af_heart}
tts_speed=${TTS_SPEED:-1.35}
say_rate=${SAY_RATE:-178}

pkg=${1:?give the demo package, for example hw/demo/fpga-icesugarpro-alphabet}
shift
preflight=0
make_images=1
while [ $# -gt 0 ]; do
    case "$1" in
        --preflight) preflight=1; shift ;;
        --no-images) make_images=0; shift ;;
        *) echo "unknown option: $1" >&2; exit 2 ;;
    esac
done

fail() { echo "$1" >&2; exit 1; }

[ -d "$pkg" ] || fail "no demo package at $pkg"
for part in narrate.sh images.sh markers.txt FORMAT.md; do
    [ -f "$pkg/$part" ] || fail "$pkg has no $part"
done

echo "--- preflight ---"
echo "package: $pkg"

command -v asciinema >/dev/null 2>&1 || fail "asciinema is not installed"
command -v agg >/dev/null 2>&1 || fail "agg is not installed, and the render needs it"
command -v cargo >/dev/null 2>&1 || fail "cargo is not installed"
command -v python3 >/dev/null 2>&1 || fail "python3 is not installed"

if [ -d /Volumes/iCELink ]; then
    echo "board: iCELink is mounted"
else
    echo "board: no iCELink volume; checking the programmer anyway"
fi

if ! "$icesprog" --help >/dev/null 2>&1; then
    fail "ICESPROG=$icesprog is not runnable"
fi
# The console shows the commands the run issues, so the programmer is named the
# way it is called: on PATH, not by an absolute path in every line.
case "$icesprog" in
    */*)
        PATH="$(cd "$(dirname "$icesprog")" && pwd):$PATH"
        export PATH
        icesprog=$(basename "$icesprog")
        ;;
esac
echo "programmer: $icesprog"

[ -d hw/synth/yosys ] || fail "hw/synth/yosys is not here; run from the repository"
echo "flow: hw/synth/yosys"

# Which checkout the recording was made against. Named rather than pinned while
# the flow is in development, so the numbers in a recording can be attributed to
# the tree they came from.
revision=$(git rev-parse --short HEAD 2>/dev/null || echo unknown)
if [ -n "$(git status --porcelain --untracked-files=no 2>/dev/null)" ]; then
    revision="$revision (dirty)"
fi
echo "syntagma: $revision"

if [ "$make_images" -eq 1 ]; then
    echo "images: they will be built, which needs nextpnr-ecp5 and ecppll"
else
    echo "images: already built"
fi

if [ "$preflight" -eq 1 ]; then
    echo
    echo "preflight passed; nothing was recorded and nothing was programmed"
    exit 0
fi

if [ "$make_images" -eq 1 ]; then
    ICESPROG="$icesprog" "$pkg/images.sh"
fi

# The take opens on whatever the part is already running, so the package puts it
# there first: the image the run refuses, whose verdict is the first thing the
# footage shows.
echo
echo "--- board: the state the take opens on ---"
ICESPROG="$icesprog" "$pkg/images.sh" --stage

stamp=$(date +%Y%m%d-%H%M%S)
out="$pkg/out/$stamp"
mkdir -p "$out"
cp "$pkg/markers.txt" "$out/markers.txt"
# The words a caption may not carry belong to the package: what sits under a
# format is not what sits under a store.
if [ -f "$pkg/leaks.txt" ]; then
    cp "$pkg/leaks.txt" "$out/leaks.txt"
fi

echo
echo "--- recording, at $styles ---"
# asciicast v2 rather than the v3 default: v3 carries relative intervals, and a
# player or an upload target that only knows v2 shows the first line and stops.
ICESPROG="$icesprog" \
    VOICE_LOG="$out/voice.log" \
    TTS="$tts" TTS_VOICE="$tts_voice" TTS_SPEED="$tts_speed" SAY_RATE="$say_rate" \
    asciinema rec -f asciicast-v2 --window-size "$styles" --overwrite \
    --command "$pkg/narrate.sh" "$out/demo.cast"

echo
echo "--- rendering ---"
./hw/demo/_shared/render.sh "$out/demo.cast" "$out/demo.mp4"

echo
echo "--- the narration track and its captions ---"
TTS="$tts" TTS_VOICE="$tts_voice" TTS_SPEED="$tts_speed" SAY_RATE="$say_rate" \
    ./hw/demo/_shared/voice.sh "$out" "$out/voice.wav" "$out/demo.srt"

echo
echo "--- checking the take ---"
if ! ./hw/demo/_shared/check.sh "$out"; then
    fail "the take at $out is not fit to show"
fi

echo
echo "cast:     $out/demo.cast"
echo "video:    $out/demo.mp4"
echo "voice:    $out/voice.wav"
echo "captions: $out/demo.srt"
