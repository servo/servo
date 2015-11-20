#!/bin/bash
./mach test-wpt --manifest-update SKIP_TESTS
diff=$(git diff --exit-code -- tests/**/MANIFEST.json)
echo "$diff"
[[ ! $diff ]]
