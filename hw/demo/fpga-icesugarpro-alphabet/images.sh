#!/usr/bin/env bash
# Build the two images the recording programs, and put one of them on the part.
#
# The input is the alphabet itself. tagma_core writes one anchor per valid
# offset, the host turns those offsets into the 16-bit values the image reads,
# and the checksum of the anchors is the number the die has to reach. The second
# image is the same set with one value moved outside the alphabet, so the refusal
# is a negative control rather than a claim the die makes about itself.
#
#   out/images/alphabet_pass.bit   every value is in the alphabet
#   out/images/alphabet_fail.bit   one value is not
#   out/images/alphabet.checksum   the sum the host computed
#
# Modes:
#   images.sh           build both images, then stage the part
#   images.sh --build   build only
#   images.sh --stage   program both, read the two verdicts, leave the refusal
#
# The values are written to the ROM path the flow fixes, because the image reads
# that file at synthesis. Nothing in the take shows that path: the ROM is an
# input to the build, and the take starts from images that already exist.

set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
hw=$(cd "$here/../.." && pwd)
icesprog=${ICESPROG:-icesprog}
images="$here/out/images"
anchors="$hw/rtl/golden_anchors.hex"
rom="$hw/synth/yosys/out/volume_rom.hex"
checksum_file="$images/alphabet.checksum"
count=11172
moved=5000
outside=11172

mode=all
case "${1:-}" in
    --build) mode=build ;;
    --stage) mode=stage ;;
    "") ;;
    *) echo "unknown option: $1" >&2; exit 2 ;;
esac

fail() { echo "$1" >&2; exit 1; }

# One 16-bit value a line, in the order the die reads them. A replacement moves
# one value outside the alphabet, which is what the second image is for.
write_rom() {
    python3 - "$rom" "$count" "$moved" "${1:-}" <<'PY'
import sys
path, count, index, replacement = sys.argv[1], int(sys.argv[2]), int(sys.argv[3]), sys.argv[4]
values = ["%04X" % value for value in range(count)]
if replacement:
    values[index] = "%04X" % int(replacement)
open(path, "w").write("\n".join(values) + "\n")
PY
}

sum_anchors() {
    python3 "$hw/tools/tagma_checksum.py" "$anchors" 2>/dev/null
}

build() {
    local bit=$1 replacement=${2:-}
    if [ -n "$replacement" ]; then
        write_rom "$replacement"
    else
        write_rom
    fi
    make --no-print-directory -C "$hw" pnr BOARD=icesugar_pro DESIGN=volume \
        EXPECTED_CHECKSUM="$checksum" VERIFY_MHZ=40
    cp "$hw/synth/yosys/out/tagma_volume_verify_top_40mhz.bit" "$bit"
    cp "$rom" "${bit%.bit}.hex"
}

stage_image() {
    "$icesprog" -w "$1" >/dev/null
    # icesprog treats -m and -w as exclusive, so the mode and the value are two
    # calls. PB7 drives the reset the image comes out of.
    "$icesprog" -g PB7 -m out >/dev/null
    "$icesprog" -g PB7 -w 0 >/dev/null
    sleep 1
    "$icesprog" -g PB7 -w 1 >/dev/null
    sleep 2
}

verdict() {
    "$icesprog" -g PA13 -m in >/dev/null
    "$icesprog" -g PA13 -r | grep -o 'return [0-9]*' | awk '{print $2}'
}

if [ "$mode" != stage ]; then
    for tool in nextpnr-ecp5 ecppll ecppack yosys; do
        command -v "$tool" >/dev/null 2>&1 ||
            fail "$tool is not on PATH; OSS CAD Suite has all four"
    done

    echo "--- the anchors: tagma_core writes them ---"
    make --no-print-directory -C "$hw" golden-export

    checksum=$(sum_anchors)
    [ -n "$checksum" ] || fail "the anchors produced no checksum"
    mkdir -p "$images"
    printf '%s\n' "$checksum" > "$checksum_file"

    echo "--- the passing image: the alphabet's own values ---"
    build "$images/alphabet_pass.bit"

    echo "--- the image with one value outside it ---"
    build "$images/alphabet_fail.bit" "$outside"

    # Leave the flow holding the alphabet's own values, so a later build without
    # this script produces the passing image rather than the changed one.
    write_rom

    echo
    echo "images: $images/alphabet_pass.bit, $images/alphabet_fail.bit"
    echo "checksum: $checksum"
fi

if [ "$mode" != build ]; then
    for file in "$images/alphabet_pass.bit" "$images/alphabet_fail.bit"; do
        [ -f "$file" ] || fail "missing $file; run without --stage to build it"
    done
    [ -f "$checksum_file" ] || fail "missing $checksum_file; run --build"
    built=$(cat "$checksum_file")
    current=$(sum_anchors)
    [ "$built" = "$current" ] ||
        fail "the images were built for $built, not $current; run --build"

    echo "--- the part: the alphabet first, then the changed set ---"
    stage_image "$images/alphabet_pass.bit"
    passed=$(verdict)
    [ "$passed" = "1" ] || fail "the alphabet image does not read a match; it reads $passed"

    stage_image "$images/alphabet_fail.bit"
    refused=$(verdict)
    [ "$refused" = "0" ] || fail "the changed image does not read a refusal; it reads $refused"

    for pin in PA0 PA1; do
        "$icesprog" -g "$pin" -m out >/dev/null
        "$icesprog" -g "$pin" -w 0 >/dev/null
    done
    echo "board: the part holds the changed set, and reads its refusal"
fi
