#!/usr/bin/env bash
REPEAT_COUNT=100
set -o errexit
set -o nounset
set -o pipefail

for test_type in wpt css; do
    while read test_name; do
        echo "  - Checking ${test_name}"
        ./mach "test-${test_type}" --log-raw - --release --repeat "${REPEAT_COUNT}" "${test_name}" > intermittents.log < /dev/null
    done < "etc/ci/former_intermittents_${test_type}.txt"
done

