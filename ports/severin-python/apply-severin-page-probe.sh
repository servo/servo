#!/usr/bin/env bash
# Apply the one-shot native page-global probe, then exec the requested build.
# Usage:
#   bash ports/severin-python/apply-severin-page-probe.sh cargo build --release -p severin-python
set -euo pipefail

root="$(git rev-parse --show-toplevel)"
patch="$root/ports/severin-python/severin-page-probe.patch"

if git -C "$root" apply --reverse --check "$patch"; then
    echo "SEVERIN_PROBE: native patch already applied"
elif git -C "$root" apply --check "$patch"; then
    git -C "$root" apply "$patch"
    echo "SEVERIN_PROBE: native patch applied"
else
    echo "SEVERIN_PROBE: patch does not match this checkout" >&2
    exit 2
fi

exec "$@"
