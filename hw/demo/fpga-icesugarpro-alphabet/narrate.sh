#!/usr/bin/env bash
# The recorded run: the alphabet, the sum, and the die's verdict.
#
# The board is not only the verdict here. The host sets a stage before each step
# and the part turns that stage into a colour, so the LED says where the run is
# as well as whether it passed. Stage 3 is the verdict, and the colour it shows
# is the die's answer, not the host's.
#
# The stage is two bits because the iCELink gives two drivable pins; its third,
# PA14 on F5, is the SWD clock and never arrives.
#
# It does not build. The two images have to exist already, and the script fails
# unless they were built for the sum these anchors produce.
#
# It is not run by hand. `hw/demo/fpga-icesugarpro-alphabet/demo.sh` records it
# at 131x31.
#
# Usage: hw/demo/fpga-icesugarpro-alphabet/narrate.sh [--no-say]

set -euo pipefail
# No job-control chatter: an async narration would print a "[1]+ Done" line in
# the middle of a take, which reads as a fault.
set +m

here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../../.." && pwd)
rel=${here#"$root"/}
cd "$root"

say_cues=1
for arg in "$@"; do
    [ "$arg" = "--no-say" ] && say_cues=0
done

icesprog=${ICESPROG:-icesprog}
images="$rel/out/images"
checksum_file="$images/alphabet.checksum"
anchors=hw/rtl/golden_anchors.hex
# The engine, the voice, and the pace are one setting: voice.sh reads the same
# three, so the track is the voice the room hears rather than a second take.
tts=${TTS:-voice}
tts_voice=${TTS_VOICE:-af_heart}
tts_speed=${TTS_SPEED:-1.35}
say_rate=${SAY_RATE:-178}
voice_log=${VOICE_LOG:-}

fail() { echo "$1" >&2; exit 1; }

# Every spoken line is stamped here, so the narration track can be built
# afterwards at the times the run actually spoke.
stamp() {
    [ -n "$voice_log" ] || return 0
    python3 -c 'import sys,time; print("%.3f\t%s" % (time.time(), sys.argv[1]))' "$1" >> "$voice_log"
}

# The function is named utter, not voice: the engine is a command of that name,
# and a function would shadow it and recurse.
utter() {
    case "$tts" in
        voice)
            command -v voice >/dev/null 2>&1 || return 0
            command voice say -q -v "$tts_voice" -s "$tts_speed" "$1" || true ;;
        say)
            command -v say >/dev/null 2>&1 || return 0
            command say -r "$say_rate" "$1" || true ;;
        none)
            return 0 ;;
        *)
            echo "unknown TTS: $tts (expected voice, say or none)" >&2
            return 1 ;;
    esac
}

spoken() {
    stamp "$1"
    [ "$say_cues" -eq 1 ] || return 0
    utter "$1"
}

# A spoken line ends where it ends, and a beat is what makes the next one a new
# thought rather than the same breath.
beat() {
    sleep "${1:-0.9}"
}

# The narration is the voice and the captions. It is not printed, because the
# console shows what the programs did: a line of prose between two commands reads
# as a script being read rather than as a run.
speak() {
    spoken "$1"
}

voiceover() {
    stamp "$1"
    [ "$say_cues" -eq 1 ] || return 0
    utter "$1" &
}

step() {
    spoken "$1"
}

# The camera cue is for the room, not for the record: it is spoken and never
# stamped, so it cannot become a caption about filming.
cue() {
    [ "$say_cues" -eq 1 ] || return 0
    utter "$1"
}

# The console is the evidence, so it carries the command as it is typed and
# whatever the tool prints, live. Nothing is captured and summarised.
say_cmd() {
    printf '$ %s\n' "$*" >&2
}

run_show() {
    say_cmd "$*"
    "$@"
}

# The stage the part turns into a colour: bit 0 on N4, bit 1 on J5.
# icesprog treats -m and -w as exclusive, so each pin takes two calls.
open_pins() {
    local pin
    for pin in PA0 PA1; do
        say_cmd "$icesprog -g $pin -m out"
        "$icesprog" -g "$pin" -m out
    done
}

set_stage() {
    local n=$1 pin bit
    for pin in PA0 PA1; do
        case "$pin" in
            PA0) bit=$(( (n >> 0) & 1 )) ;;
            PA1) bit=$(( (n >> 1) & 1 )) ;;
        esac
        say_cmd "$icesprog -g $pin -w $bit"
        "$icesprog" -g "$pin" -w "$bit"
    done
}

# The LED is what the board shows, so the console states it where the stage
# changes, in the colour the board is showing.
led() {
    printf '\033[1;%sm>>> [LED] %s\033[0m\n' "$1" "$2"
}

# The read is the device's answer, so its own two lines are shown. The value goes
# to stdout for the caller and the transcript goes to the terminal.
read_pin() {
    local pin=$1 mode answer value
    say_cmd "$icesprog -g $pin -m in"
    mode=$("$icesprog" -g "$pin" -m in)
    printf '%s\n' "$mode" >&2
    say_cmd "$icesprog -g $pin -r"
    answer=$("$icesprog" -g "$pin" -r)
    printf '%s\n' "$answer" >&2
    value=$(printf '%s' "$answer" | grep -o 'return [0-9]*' | awk '{print $2}')
    printf '%s\n' "$value"
}

# Writing a bitstream takes about a minute, and the programmer reports every
# 64 kB it lands, so the screen moves for as long as the write does.
load() {
    local bit=$1
    run_show "$icesprog" -w "$bit"
    run_show "$icesprog" -g PB7 -m out
    run_show "$icesprog" -g PB7 -w 0
    sleep 1
    run_show "$icesprog" -g PB7 -w 1
    sleep 2
}

for file in "$images/alphabet_pass.bit" "$images/alphabet_fail.bit" "$checksum_file"; do
    [ -f "$file" ] || fail "missing $file; build it with $rel/images.sh"
done
built=$(cat "$checksum_file")
current=$(python3 hw/tools/tagma_checksum.py "$anchors" 2>/dev/null)
[ "$built" = "$current" ] ||
    fail "the images were built for $built, not $current; a take cannot show another run's verdict"

# The take opens on an empty console, so the recording gives the terminal a
# moment before the first command.
cue "Start filming."
sleep 3

open_pins
# One stage covers both host acts, because the footage this take is cut against
# carries one colour between its start and the red. A film of this script would
# let the two acts keep their own stages; the stage table in FORMAT.md is the
# vocabulary the top can show either way.
set_stage 1
led 96 "cyan, blink"
step "The alphabet."
speak "Tagma writes one anchor for every valid offset of its alphabet."
run_show make --no-print-directory -C hw golden-export
speak "Eleven thousand one hundred and seventy two of them, each a code point that packs three axes."

step "The sum."
speak "The host sums those packs, and that number is what the die has to reach."
run_show python3 hw/tools/tagma_checksum.py "$anchors"
run_show make --no-print-directory -C hw golden-check
speak "The anchors pass their own gate before anything is built from them."

step "A set with one value outside the alphabet."
match=$(read_pin PA13)
set_stage 3
led 91 "red, steady"
echo "pin_match = $match"
[ "$match" = "0" ] || fail "the refusal did not happen"
speak "Red."
beat 0.9
speak "One value moved to eleven thousand one hundred and seventy two, which the format cannot hold, and the die refused the set."
beat 1.2

set_stage 2
led 93 "yellow, blink"
step "The die is given the alphabet."
speak "Now the alphabet's own values, written to the part."
load "$images/alphabet_pass.bit"
match=$(read_pin PA13)
set_stage 3
led 92 "green, steady"
echo "pin_match = $match"
[ "$match" = "1" ] || fail "the part did not reach $built"
speak "Green."
beat 0.9
speak "The die decoded every value and reached the host's number, and it shares no code with the reference."
beat 1.2

speak "A format a part can check is a format two implementations can agree on."
beat 1.2

cue "Stop filming."
sleep 4
set_stage 0
