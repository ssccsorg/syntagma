#!/usr/bin/env bash
# Record this package. One line by design: the recorder is shared, the story is
# here, and the takes land in ./out.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../../.." && pwd)
exec "$root/hw/demo/_shared/run.sh" "${here#"$root"/}" "$@"
