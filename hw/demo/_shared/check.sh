#!/usr/bin/env bash
# Decide whether a take is fit to show, before anyone shows it.
#
# A take is one console recording of the whole run, so a take that stops after
# the first line is not a short demo, it is a broken one. This reads the cast,
# extracts the absolute time of every step marker, and refuses the take unless
# each marker is present, in order, and the video that was rendered from it has
# the same length. run.sh calls it after rendering, so a bad take fails loudly
# instead of being handed to someone.
#
# Usage: demo/_shared/check.sh OUTDIR [--min SECONDS] [--tolerance SECONDS]

set -euo pipefail
cd "$(dirname "$0")/../../.."

out=${1:?give the take directory, for example demo/fpga-icesugarpro-keyspace/out/20261114-105238}
shift || true

min_seconds=120
tolerance=5
while [ $# -gt 0 ]; do
    case "$1" in
        --min)       min_seconds=$2; shift 2 ;;
        --tolerance) tolerance=$2;   shift 2 ;;
        *) echo "unknown argument: $1" >&2; exit 2 ;;
    esac
done

cast="$out/demo.cast"
video="$out/demo.mp4"
voice="$out/voice.wav"
captions="$out/demo.srt"
markers_file="$out/markers.txt"
leaks_file="$out/leaks.txt"

[ -f "$cast" ] || { echo "missing $cast" >&2; exit 1; }
[ -f "$markers_file" ] || { echo "missing $markers_file; a take carries the markers of its scenario" >&2; exit 1; }

python3 - "$cast" "$video" "$voice" "$captions" "$markers_file" "$min_seconds" "$tolerance" \
         "$leaks_file" <<'PY'
import json, os, re, subprocess, sys

(cast, video, voice, captions, markers_file, min_seconds, tolerance,
 leaks_file) = (sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4], sys.argv[5],
                float(sys.argv[6]), float(sys.argv[7]), sys.argv[8])


def probe(path):
    try:
        return float(subprocess.run(
            ["ffprobe", "-v", "error", "-show_entries", "format=duration",
             "-of", "csv=p=0", path],
            capture_output=True, text=True, check=True).stdout.strip())
    except Exception:
        return None

markers = []
with open(markers_file, encoding="utf-8") as handle:
    for line in handle:
        line = line.strip()
        if line and not line.startswith("#"):
            markers.append(line)
if not markers:
    print(f"FAIL: {markers_file} holds no marker")
    sys.exit(1)

# asciicast v3 carries an interval per event and v2 an absolute time, so the
# header decides how the clock is accumulated.
version = 2
clock = 0.0
events = 0
seen = {}
with open(cast) as handle:
    for line in handle:
        line = line.strip()
        if not line:
            continue
        if line.startswith("{"):
            version = json.loads(line).get("version", 2)
            continue
        if not line.startswith("["):
            continue
        entry = json.loads(line)
        if len(entry) < 3 or entry[1] != "o":
            continue
        clocks = float(entry[0])
        clock = clock + clocks if version >= 3 else clocks
        events += 1
        for marker in markers:
            if marker in entry[2] and marker not in seen:
                seen[marker] = clock

failures = []

print(f"cast   {cast} (asciicast v{version})")
print(f"events {events}, last output at {clock:.1f}s")
print()

last_time = -1.0
for marker in markers:
    when = seen.get(marker)
    if when is None:
        failures.append(f"missing: {marker}")
        print(f"  {'MISSING':>8}  {marker}")
        continue
    if when < last_time:
        failures.append(f"out of order: {marker} at {when:.1f}s, after {last_time:.1f}s")
        print(f"  {'ORDER':>8}  {int(when//60)}:{when%60:05.2f}  {marker}")
        continue
    last_time = when
    print(f"  {'ok':>8}  {int(when//60)}:{when%60:05.2f}  {marker}")

if clock < min_seconds:
    failures.append(f"the cast ends at {clock:.1f}s, below the {min_seconds:.0f}s a full run takes")
    print(f"\n  {'SHORT':>8}  last output at {clock:.1f}s, expected at least {min_seconds:.0f}s")

if not os.path.exists(video):
    failures.append(f"missing {video}")
    print(f"\n  {'MISSING':>8}  {video}")
else:
    duration = probe(video)
    if duration is None:
        failures.append(f"cannot read the duration of {video}")
    else:
        delta = abs(duration - clock)
        state = "ok" if delta <= tolerance else "LENGTH"
        print(f"  {state:>8}  video {duration:.1f}s against cast {clock:.1f}s")
        if delta > tolerance:
            failures.append(
                f"the video is {duration:.1f}s but the cast ends at {clock:.1f}s "
                f"(tolerance {tolerance:.0f}s), so the render lost the run")

if not os.path.exists(voice):
    failures.append(f"missing {voice}")
    print(f"\n  {'MISSING':>8}  {voice}")
else:
    spoken = probe(voice)
    if spoken is None:
        failures.append(f"cannot read the duration of {voice}")
    else:
        delta = abs(spoken - duration) if duration else abs(spoken - clock)
        state = "ok" if delta <= tolerance else "LENGTH"
        print(f"  {state:>8}  voice {spoken:.1f}s against the video {duration:.1f}s")
        if delta > tolerance:
            failures.append(
                f"the spoken track is {spoken:.1f}s against a {duration:.1f}s video, "
                "so it would not stay in step")

if not os.path.exists(captions):
    failures.append(f"missing {captions}")
    print(f"\n  {'MISSING':>8}  {captions}")
else:
    with open(captions, encoding="utf-8") as handle:
        body = handle.read()
    pairs = []
    for match in re.finditer(
            r"(\d+):(\d+):(\d+),(\d+) --> (\d+):(\d+):(\d+),(\d+)", body):
        groups = [int(value) for value in match.groups()]
        pairs.append((groups[:4], groups[4:]))
    cues = len(pairs)
    if not pairs:
        failures.append(f"{captions} holds no cue")
        print(f"\n  {'EMPTY':>8}  {captions}")
    else:
        def seconds(parts):
            return parts[0] * 3600 + parts[1] * 60 + parts[2] + parts[3] / 1000

        last_seconds = seconds(pairs[-1][1])
        limit = (duration if duration else clock) + 0.1
        state = "ok" if last_seconds <= limit else "LENGTH"
        print(f"  {state:>8}  captions {cues} cues, last at {last_seconds:.1f}s")
        if last_seconds > limit:
            failures.append(
                f"the last cue ends at {last_seconds:.1f}s, past the {limit:.1f}s of the take")

        # A promotional demo says what the thing itself does. A word from the
        # layer under it, or from whatever would consume it, is a leak, and it
        # is how the principle creeps back in. The package lists its own words,
        # because what is under a format is not the same as what is under a store.
        forbidden = []
        if os.path.exists(leaks_file):
            with open(leaks_file, encoding="utf-8") as handle:
                for line in handle:
                    word = line.strip()
                    if word and not word.startswith("#"):
                        forbidden.append(word)
        leaked = [
            word for word in forbidden
            if re.search(re.escape(word), body, re.IGNORECASE)
        ]
        if forbidden:
            state = "ok" if not leaked else "LEAK"
            print(f"  {state:>8}  captions carry none of the {len(forbidden)} words the package forbids")
            if leaked:
                failures.append(
                    f"the captions carry {', '.join(leaked)}, which this package forbids")
        else:
            print(f"  {'none':>8}  the package forbids no word, so this was not checked")

        # A sentence read straight into the next one is what a viewer hears as
        # rushed, so the gaps between cues are part of the take's quality.
        gaps = [seconds(pairs[i + 1][0]) - seconds(pairs[i][1]) for i in range(len(pairs) - 1)]
        tightest = min(gaps) if gaps else None
        if tightest is not None:
            state = "ok" if tightest >= 0.6 else "TIGHT"
            print(f"  {state:>8}  tightest gap between cues {tightest:.2f}s")
            if tightest < 0.6:
                failures.append(
                    f"two cues are {tightest:.2f}s apart, which reads as one sentence")

print()
if failures:
    for failure in failures:
        print(f"FAIL: {failure}")
    print("\nthe take is not fit to show")
    sys.exit(1)

print("PASS: every step is in one console recording, and the video, the voice "
      "and the captions have its length")
PY
