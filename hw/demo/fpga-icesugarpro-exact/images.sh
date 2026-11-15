#!/usr/bin/env bash
# Build the two images the recording programs, and put one of them on the part.
#
# The input is the anchor file tagma_core writes. Two files are generated from
# it: the lines the reference wrote, and the same lines with two packs exchanged.
# The sums of the two are equal, which is the point. The image that compares a sum
# cannot tell them apart, and this one must.
#
#   out/images/anchor_pass.bit   the lines the reference wrote
#   out/images/anchor_swap.bit   the same lines, two packs exchanged
#   out/images/anchor_run.json   the artefacts, and the two verdicts read off PA13
#
# Modes:
#   images.sh           build both images, then stage the part
#   images.sh --build   build only
#   images.sh --stage   program both, read the two verdicts, leave the swapped one
#
# The take opens on the image the run refuses, so --stage leaves the swapped one
# on the part. The record is written there, because that is when both verdicts
# are observed.

set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
hw=$(cd "$here/../.." && pwd)
icesprog=${ICESPROG:-icesprog}
images="$here/out/images"
anchors="$hw/rtl/golden_anchors.hex"
fixtures="$hw/synth/yosys/out"
record="$images/anchor_run.json"
flow="$hw/synth/yosys/out"
mhz=40
# The two lines whose packs are exchanged. The generator and the record take it
# from here, so the artefact and the note about it cannot disagree.
swap=2,5

mode=all
case "${1:-}" in
    --build) mode=build ;;
    --stage) mode=stage ;;
    "") ;;
    *) echo "unknown option: $1" >&2; exit 2 ;;
esac

fail() { echo "$1" >&2; exit 1; }

sum_of() {
    python3 "$hw/tools/tagma_checksum.py" "$1" 2>/dev/null
}

build() {
    local tag=$1
    ANCHOR_ROM="out/anchor_fixture_${tag}.hex" ANCHOR_TAG="$tag" \
        make --no-print-directory -C "$hw" pnr BOARD=icesugar_pro DESIGN=anchor VERIFY_MHZ="$mhz"
    cp "$flow/tagma_anchor_verify_top_${mhz}mhz_${tag}.bit" "$images/anchor_${tag}.bit"
}

stage_image() {
    "$icesprog" -w "$1" >/dev/null
    # icesprog treats -m and -w as exclusive, so the mode and the value are two
    # calls. PB7 drives the reset the image comes out of.
    "$icesprog" -g PB7 -m out >/dev/null
    "$icesprog" -g PB7 -w 0 >/dev/null
    sleep 1
    "$icesprog" -g PB7 -w 1 >/dev/null
    # The part answers about two seconds after it is released, so a shorter wait
    # reads the verdict of the image it is still replacing. Measured: the pass
    # image reads 0 until roughly 2 s, then 1 for every read after.
    sleep 6
}

verdict() {
    "$icesprog" -g PA13 -m in >/dev/null
    "$icesprog" -g PA13 -r | grep -o 'return [0-9]*' | awk '{print $2}'
}

write_record() {
    local passed=$1 refused=$2
    python3 - "$record" "$anchors" "$fixtures/anchor_fixture_pass.hex" \
        "$fixtures/anchor_fixture_swap.hex" "$images/anchor_pass.bit" \
        "$images/anchor_swap.bit" "$mhz" "$hw" "$swap" "$passed" "$refused" <<'PY'
import hashlib, json, subprocess, sys

(record, anchors, pass_hex, swap_hex, pass_bit, swap_bit, mhz, hw, swap,
 passed, refused) = sys.argv[1:12]


def digest(path):
    h = hashlib.sha256()
    with open(path, "rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            h.update(block)
    return h.hexdigest()


def lines(path):
    with open(path, encoding="utf-8") as handle:
        return sum(1 for line in handle if line.strip())


def revision():
    try:
        return subprocess.run(
            ["git", "-C", hw, "rev-parse", "--short", "HEAD"],
            capture_output=True, text=True, check=True).stdout.strip()
    except Exception:
        return "unknown"


data = {
    "package": "fpga-icesugarpro-exact",
    "design": "tagma_anchor_verify_top",
    "board": "iCESugar-Pro v1.3 (ECP5-25F)",
    "clock_mhz": int(mhz),
    "anchors": {
        "path": "hw/rtl/golden_anchors.hex",
        "sha256": digest(anchors),
        "lines": lines(anchors),
    },
    "fixtures": {
        "pass": {"path": "synth/yosys/out/anchor_fixture_pass.hex",
                 "sha256": digest(pass_hex)},
        "swap": {"path": "synth/yosys/out/anchor_fixture_swap.hex",
                 "sha256": digest(swap_hex),
                 "packs_exchanged": [int(part) for part in swap.split(",")]},
    },
    "bitstreams": {
        "pass": {"sha256": digest(pass_bit), "observed": int(passed)},
        "swap": {"sha256": digest(swap_bit), "observed": int(refused)},
    },
    "observation": {"pin": "PA13", "pass_expected": 1, "swap_expected": 0},
    "syntagma_revision": revision(),
}
with open(record, "w", encoding="utf-8") as handle:
    json.dump(data, handle, indent=2, sort_keys=True)
    handle.write("\n")
PY
}

if [ "$mode" != stage ]; then
    for tool in nextpnr-ecp5 ecppll ecppack yosys; do
        command -v "$tool" >/dev/null 2>&1 ||
            fail "$tool is not on PATH; OSS CAD Suite has all four"
    done

    echo "--- the anchors: tagma_core writes them ---"
    make --no-print-directory -C "$hw" golden-export

    mkdir -p "$images"

    echo "--- the pair: the anchors, and the same anchors with two packs exchanged ---"
    python3 "$hw/tools/anchor_fixtures.py" --anchors "$anchors" --swap "$swap" "$fixtures"

    pass_sum=$(sum_of "$fixtures/anchor_fixture_pass.hex")
    swap_sum=$(sum_of "$fixtures/anchor_fixture_swap.hex")
    [ -n "$pass_sum" ] || fail "the pass fixture produced no sum"
    [ "$pass_sum" = "$swap_sum" ] ||
        fail "the fixtures read $pass_sum and $swap_sum, so the pair is not a control"

    echo "--- the image the reference's lines build ---"
    build pass

    echo "--- the image whose two packs moved ---"
    build swap

    echo
    echo "images: $images/anchor_pass.bit, $images/anchor_swap.bit"
    echo "sum:    $pass_sum, for both files"
fi

if [ "$mode" != build ]; then
    for file in "$images/anchor_pass.bit" "$images/anchor_swap.bit"; do
        [ -f "$file" ] || fail "missing $file; run without --stage to build it"
    done
    [ -f "$fixtures/anchor_fixture_pass.hex" ] || fail "missing fixtures; run --build"

    echo "--- the part: the exchanged lines first, then the reference's ---"
    stage_image "$images/anchor_pass.bit"
    passed=$(verdict)
    [ "$passed" = "1" ] || fail "the reference's lines do not read a match; they read $passed"

    stage_image "$images/anchor_swap.bit"
    refused=$(verdict)
    [ "$refused" = "0" ] || fail "the exchanged lines do not read a refusal; they read $refused"

    for pin in PA0 PA1; do
        "$icesprog" -g "$pin" -m out >/dev/null
        "$icesprog" -g "$pin" -w 0 >/dev/null
    done

    write_record "$passed" "$refused"

    echo "board: the part holds the exchanged lines, and reads its refusal"
    echo "record: $record"
fi
