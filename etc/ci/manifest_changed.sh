#!/bin/bash
set -o errexit
set -o nounset
set -o pipefail

# We shouldn't need any binary at all to update the manifests.
# Adding "SKIP_TESTS" to skip tests, it doesn't really skip the tests.
# It will run "run_wpt" with "'test_list': ['SKIP_TESTS']", and then pass it into wptrunner.
# Adding "--binary=" to skip binary check.
./mach test-wpt --manifest-update --binary= SKIP_TESTS > /dev/null

diff=$(git diff -- tests/*/MANIFEST.json)
echo "$diff"
[[ ! $diff ]]
