#!/usr/bin/env bash
# Build the spoken track for a take, from the lines the run actually spoke.
#
# narrate.sh stamps every spoken line with the wall clock at the moment it
# spoke. The cast records the same run on its own clock, so the first stamped
# line is matched against the first output in the cast to put the two on one
# origin; from there every line lands where the recording heard it. Each line is
# then synthesised once at the rate the run used and mixed at its offset, so the
# result is the same voice the room heard, on the timeline of the silent video.
#
# Usage: demo/_shared/voice.sh OUTDIR [TRACK] [CAPTIONS]

set -euo pipefail
cd "$(dirname "$0")/../../.."

out=${1:?give the take directory}
track=${2:-$out/voice.wav}
captions=${3:-$out/demo.srt}

say_rate=${SAY_RATE:-178}
tts=${TTS:-voice}
tts_voice=${TTS_VOICE:-af_heart}
tts_speed=${TTS_SPEED:-1.35}
log="$out/voice.log"
cast="$out/demo.cast"
video="$out/demo.mp4"

[ -f "$log" ] || { echo "$log does not exist; the run did not stamp its lines" >&2; exit 1; }
[ -f "$cast" ] || { echo "$cast does not exist" >&2; exit 1; }
[ -f "$video" ] || { echo "$video does not exist; render it first" >&2; exit 1; }
case "$tts" in
    voice) command -v voice >/dev/null 2>&1 || { echo "voice is not installed" >&2; exit 1; } ;;
    say)   command -v say >/dev/null 2>&1 || { echo "say is not available" >&2; exit 1; } ;;
esac

length=$(ffprobe -v error -show_entries format=duration -of csv=p=0 "$video")
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

python3 - "$log" "$cast" "$length" "$track" "$work" \
         "$tts" "$tts_voice" "$tts_speed" "$say_rate" "$captions" <<'PY'
import json, os, subprocess, sys, textwrap, wave
from array import array

(log, cast, length, track, work, engine, tts_voice, tts_speed, say_rate,
 captions) = sys.argv[1:11]
length = float(length)

def synth(text, path):
    if engine == "voice":
        raw = path + ".raw.wav"
        subprocess.run(
            ["voice", "say", "-q", "-v", tts_voice, "-s", tts_speed, "-o", raw, text],
            check=True, capture_output=True)
        # voice writes 24 kHz mono float32, and the mix below is 16-bit PCM.
        subprocess.run(
            ["ffmpeg", "-v", "error", "-y", "-i", raw, "-ar", "44100", "-ac", "1",
             "-c:a", "pcm_s16le", path],
            check=True, capture_output=True)
        os.remove(raw)
    else:
        subprocess.run(
            ["say", "-r", say_rate, "-o", path, "--data-format=LEI16@44100", text],
            check=True, capture_output=True)

rate_hz = 44100
frames = int(round(length * rate_hz))

lines = []
with open(log) as handle:
    for entry in handle:
        entry = entry.rstrip("\n")
        if not entry:
            continue
        when, text = entry.split("\t", 1)
        lines.append((float(when), text))
if not lines:
    print("no spoken lines were stamped")
    sys.exit(1)

# asciicast v2 timestamps the recording's own zero, and every event time is
# relative to it, so the header's epoch and the stamped wall clock share one
# origin. The fallback matches the first output against the first line, which is
# only right when the run echoes its narration.
start_epoch = None
first_output = None
with open(cast) as handle:
    for line in handle:
        line = line.strip()
        if line.startswith("{"):
            start_epoch = json.loads(line).get("timestamp")
            continue
        if not line.startswith("["):
            continue
        entry = json.loads(line)
        if len(entry) >= 3 and entry[1] == "o":
            first_output = float(entry[0])
            break

if start_epoch:
    origin = float(start_epoch)
elif first_output is not None:
    origin = lines[0][0] - first_output
else:
    origin = lines[0][0]

buffer = array("h", bytes(2 * frames))
placed = 0
cues = []

# A synthesised line carries silence before and after the words. The cue has to
# cover the words, or the caption outlives the sentence and the gap to the next
# one measures shorter than the room heard it.
def trim(samples, threshold=300, margin=0.03):
    first, last = None, None
    for index, value in enumerate(samples):
        if abs(value) > threshold:
            if first is None:
                first = index
            last = index
    if first is None:
        return samples
    margin_frames = int(margin * rate_hz)
    start = max(0, first - margin_frames)
    end = min(len(samples), last + margin_frames + 1)
    return samples[start:end]


for index, (when, text) in enumerate(lines):
    offset = max(0.0, when - origin)
    path = os.path.join(work, f"line{index}.wav")
    synth(text, path)
    with wave.open(path) as handle:
        samples = array("h")
        samples.frombytes(handle.readframes(handle.getnframes()))
    if sys.byteorder == "big":
        samples.byteswap()
    samples = trim(samples)
    start_frame = int(round(offset * rate_hz))
    cues.append((start_frame / rate_hz, len(samples) / rate_hz, text))
    for position, value in enumerate(samples):
        target = start_frame + position
        if target >= frames:
            break
        mixed = buffer[target] + value
        buffer[target] = 32767 if mixed > 32767 else (-32768 if mixed < -32768 else mixed)
    placed += 1

if sys.byteorder == "big":
    buffer.byteswap()
with wave.open(track, "w") as handle:
    handle.setnchannels(1)
    handle.setsampwidth(2)
    handle.setframerate(rate_hz)
    handle.writeframes(buffer.tobytes())

# Captions come from the same two numbers as the audio, so a cue starts when its
# line was spoken and ends when that line stops sounding. A cue never runs into
# the next one or past the video.
def stamp(seconds):
    total = int(round(seconds * 1000))
    hours, total = divmod(total, 3600000)
    minutes, total = divmod(total, 60000)
    whole, millis = divmod(total, 1000)
    return f"{hours:02d}:{minutes:02d}:{whole:02d},{millis:03d}"

written = 0
with open(captions, "w") as handle:
    for index, (start, duration, text) in enumerate(cues):
        end = start + duration
        if index + 1 < len(cues):
            end = min(end, cues[index + 1][0] - 0.05)
        end = min(end, length)
        if end <= start:
            end = start + 0.3
        wrapped = textwrap.wrap(text, 42) or [text]
        handle.write(f"{index + 1}\n")
        handle.write(f"{stamp(start)} --> {stamp(end)}\n")
        handle.write("\n".join(wrapped) + "\n\n")
        written += 1

print(f"lines   {placed}")
print(f"origin  {origin:.3f}, the cast starts at its own zero")
print(f"track   {track}")
print(f"captions {captions}, {written} cues")
print(f"length  {frames / rate_hz:.2f}s, the video is {length:.2f}s")
PY

# The engines do not agree on level, and a demo track that has to be mixed with
# footage should not depend on which one spoke it, so the mix is normalised once.
ffmpeg -v error -y -i "$track" \
    -af loudnorm=I=-16:TP=-1.5:LRA=11 \
    -ar 44100 -ac 1 -c:a pcm_s16le "$track.norm.wav"
mv "$track.norm.wav" "$track"

ffprobe -v error -show_entries format=duration -of default=nw=1 "$track"
