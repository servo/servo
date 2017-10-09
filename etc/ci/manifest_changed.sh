#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

set -o errexit
set -o nounset
set -o pipefail

echo "About to update manifest."

# We shouldn't need any binary at all to update the manifests.
# Adding "SKIP_TESTS" to skip tests, it doesn't really skip the tests.
# It will run "run_wpt" with "'test_list': ['SKIP_TESTS']",
# and then pass it into wptrunner, which won't be able to find any tests named
# "SKIP_TESTS", and thus won't run any.
# Adding "--binary=" to skip looking for a compiled servo binary.
./mach test-wpt --manifest-update --binary= SKIP_TESTS

echo "Updated manifest; about to check if any changes were made to it."

diff="$(git diff -- tests/*/MANIFEST.json)"
echo "${diff}"
[[ -z "${diff}" ]]
