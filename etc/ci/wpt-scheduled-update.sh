#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

set -o errexit
set -o nounset
set -o pipefail

# For a given chunk, use the existing log files to update the expected test
# results and amend the last commit with the new results.
function unsafe_update_metadata_chunk() {
    ./mach update-wpt \
        "wpt-logs-linux-layout-2013/test-wpt.${1}.log" || return 1
    ./mach update-wpt --layout-2020 \
        "wpt-logs-linux-layout-2020/test-wpt.${1}.log" || return 2

    # Ensure any new directories or ini files are included in these changes.
    git add tests/wpt/meta \
        tests/wpt/meta-legacy-layout \
        tests/wpt/mozilla/meta \
        tests/wpt/mozilla/meta-legacy-layout || return 3

    # Merge all changes with the existing commit.
    git commit -a --amend --no-edit || return 3
}

function update_metadata_chunk() {
    unsafe_update_metadata_chunk "${1}" || \
        { code="${?}"; return "${code}"; }
}

function main() {
    for n in $(seq 1 "${MAX_CHUNK_ID}")
    do
        code=""
        update_metadata_chunk "${n}" || code="${?}"
        if [[ "${code}" != "" ]]; then
            return "${code}"
        fi
    done
}

main
