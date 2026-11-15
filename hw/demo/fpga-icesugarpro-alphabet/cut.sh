#!/usr/bin/env bash
# Cut this package's take, with the two events this board provides.
#
# The parent format is `hw/demo/FORMAT.md`, and what this board adds to it is the
# `FORMAT.md` beside this script. That one states both anchors and the times the
# footage was read at; the defaults below are those times.
#
# Usage: hw/demo/fpga-icesugarpro-alphabet/cut.sh TAKE FOOTAGE [OUT]
#
# The take is given from the repository root, for example
# hw/demo/fpga-icesugarpro-alphabet/out/20261115-120000.

set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../../.." && pwd)

exec env \
    ANCHOR_A='[LED] red, steady' \
    ANCHOR_B='[LED] green, steady' \
    FOOTAGE_A="${FOOTAGE_A:-73.50}" \
    FOOTAGE_B="${FOOTAGE_B:-151.75}" \
    "$root/hw/demo/_shared/final.sh" "$@"
