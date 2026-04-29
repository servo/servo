#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

set -euo pipefail

# This is a linker wrapper script that is intended to run on Linux CI systems,
# and serialize linker invocations by rustc to avoid out of memory issues.
# We assume that `clang` is in PATH and the linker driver. Rustc chooses `cc`
# by default.
# The wrapper can be used in CI by setting the following environment variabel:
# CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER: $GITHUB_WORKSPACE/support/linux/linker-wrapper.sh
#
# If future usecases require a custom linker driver, we could pass the real linker through
# to this script via another environment variable, perhaps via mach.
# Maybe it could make sense to integrate this wrapper into mach, to also prevent OOM errors
# during local development.

real_linker_driver=clang

if ! command -v flock >/dev/null 2>&1; then
    echo "linker-wrapper.sh: Error: flock is required to manage concurrency" >&2
    echo "linker-wrapper.sh: Invoking the linker without managing concurrency" >&2
    # Todo: Testing only, flock should be available, but lets be sure.
    exit 1
    # "${real_linker_driver}" "$@"
fi

lock_file="${TMPDIR:-/tmp}/servo-linker.lock"
lock_fd=""

cleanup() {
    if [[ -n "${lock_fd}" ]]; then
        exec {lock_fd}>&-
    fi
}

trap cleanup EXIT

# This opens $lock_file for writing, and puts the fd into lock_fd
exec {lock_fd}>"${lock_file}"

if flock "${lock_fd}"; then
    "${real_linker_driver}" "$@"
    exit $?
else
  echo "linker-wrapper.sh: Failed to flock ${lock_file}." >&2
  exit 1
fi
