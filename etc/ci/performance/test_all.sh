#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

set -o errexit
set -o nounset
set -o pipefail

base="http://localhost:8000"

while (( "${#}" ))
do
case "${1}" in
  --servo)
    engine="--engine=servo"
    timeout=60
    ;;
  --gecko)
    engine="--engine=gecko"
    timeout=15
    ;;
  --submit)
    submit=1
    ;;
  --base)
    base="${2}"
    shift
    ;;
  *)
    echo "Unknown option ${1}."
    exit
    ;;
esac
shift
done

if [[ -z "${engine:-}" ]];
then echo "You didn't specify the engine to run: --servo or --gecko."; exit;
fi

echo "Starting the local server"
python3 -m http.server > /dev/null 2>&1 &

# Stop the local server no matter how we exit the script
trap 'kill $(jobs -pr)' SIGINT SIGTERM EXIT

# TODO: enable the full manifest when #11087 is fixed
# https://github.com/servo/servo/issues/11087
# MANIFEST="page_load_test/tp5n/20160509.manifest"
MANIFEST="page_load_test/test.manifest" # A manifest that excludes
                                        # timeout test cases
PERF_KEY="perf-$(uname -s)-$(uname -m)-$(date +%s).csv"
PERF_FILE="output/${PERF_KEY}"

echo "Running tests"
python3 runner.py ${engine} --runs 4 --timeout "${timeout}" --base "${base}" \
  "${MANIFEST}" "${PERF_FILE}"

if [[ "${submit:-}" ]];
then
    echo "Submitting to S3"
    python3 submit_to_s3.py "${PERF_FILE}" "${PERF_KEY}"
fi

