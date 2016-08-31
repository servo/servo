#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

set -o errexit
set -o nounset
set -o pipefail

upload_perfherder=false
for i in "${@}"; do
case "${i}" in
  --servo)
    engine="--engine=servo"
    ;;
  --gecko)
    engine="--engine=gecko"
    ;;
  --upload)
    upload_perfherder=true
    ;;
  *)
    # This branch should never be reached with set -o nounset
    echo "You didn't specify the engine to run."
    exit
    ;;
esac
done

echo "Starting the local server"
python3 -m http.server > /dev/null 2>&1 &

# TODO: enable the full manifest when #11087 is fixed
# https://github.com/servo/servo/issues/11087
# MANIFEST="page_load_test/test.manifest"
MANIFEST="page_load_test/tp5n/20160509.manifest" # A manifest that excludes
                                                 # timeout test cases
PERF_FILE="output/perf-$(date +%s).json"

echo "Running tests"
python3 runner.py "${engine}" --runs 3 "${MANIFEST}" "${PERF_FILE}"

if "${upload_perfherder}"; then
  echo "Submitting to Perfherder"
  # Perfherder SSL check will fail if time is not accurate,
  # sync time before you submit
  # TODO: we are using Servo's revision hash for Gecko's result to make both
  # results appear on the same date. Use the correct result when Perfherder
  # allows us to change the date.
  python3 submit_to_perfherder.py "${engine}" "${PERF_FILE}" servo/revision.json
fi

# Kill the http server
trap 'kill $(jobs -pr)' SIGINT SIGTERM EXIT
