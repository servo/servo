#!/bin/bash
./mach test-wpt --manifest-update SKIP_TESTS > /dev/null
diff=$(git diff -- tests/*/MANIFEST.json)
echo "$diff"
[[ ! $diff ]]
