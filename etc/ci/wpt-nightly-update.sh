#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

set -o errexit
set -o nounset
set -o pipefail

# Using an existing log file, update the expected test results and amend the
# last commit with the new results.
function unsafe_update_metadata() {
    ./mach update-wpt "${1}" || return 1
    # Hope that any test result changes from layout-2013 are
    # also applicable to layout-2020.
    ./mach update-wpt --layout-2020 "${1}" || return 2
    # Ensure any new directories or ini files are included in these changes.
    git add tests/wpt/metadata tests/wpt/metadata-layout-2020 \
        tests/wpt/mozilla/meta || return 3
    # Merge all changes with the existing commit.
    git commit -a --amend --no-edit || return 3
}

function update_metadata() {
    unsafe_update_metadata "${1}" || \
        { code="${?}"; cleanup; return "${code}"; }
}

function main() {
    for n in $(seq 1 "${MAX_CHUNK_ID}")
    do
        code=""
        update_metadata "wpt${n}-logs-linux/test-wpt.${n}.log" || \
            code="${?}"
        if [[ "${code}" != "" ]]; then
            return "${code}"
        fi
    done
}

# Ensure we clean up after ourselves if this script is interrupted.
trap 'cleanup' SIGINT SIGTERM
main
