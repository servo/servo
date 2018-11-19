#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

set -o errexit
set -o nounset
set -o pipefail
REPEAT_COUNT=100

while read test_name; do
    echo "  - Checking ${test_name}"
    ./mach test-wpt \
        --release \
        --log-raw - \
        --repeat "${REPEAT_COUNT}" \
        "${test_name}" \
        > intermittents.log \
        < /dev/null
done < "etc/ci/former_intermittents_wpt.txt"

