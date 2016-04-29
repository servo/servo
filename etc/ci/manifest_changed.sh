#!/bin/bash
set -o errexit
set -o nounset
set -o pipefail

if `./mach test-wpt --manifest-update SKIP_TESTS > /dev/null`
then
    ./mach test-wpt --release --manifest-update SKIP_TESTS > /dev/null
fi
diff=$(git diff -- tests/*/MANIFEST.json)
echo "$diff"
[[ ! $diff ]]
