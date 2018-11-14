#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

set -o errexit
set -o nounset
set -o pipefail

TP5N_SOURCE="https://github.com/rwood-moz/talos-pagesets/raw/master/tp5n.zip"
TP5N_PATH="page_load_test/tp5n.zip"

if [[ ! -f "$(dirname "${TP5N_PATH}")/tp5n/tp5n.manifest" ]]; then
    if [[ ! -f "${TP5N_PATH}" ]]; then
        echo "Downloading the test cases..."
        wget "${TP5N_SOURCE}" -O "${TP5N_PATH}"
        echo "done"
    else
        echo "Found existing test cases, skipping download."
    fi
    echo -n "Unzipping the test cases..."
    unzip -q -o "${TP5N_PATH}" -d "$(dirname "${TP5N_PATH}")"
    echo "done"
else
    echo "Found existing test cases, skipping download and unzip."
fi

# We use the https URL for the repo so the clone works even if
# github.com isn't in ssh's known hosts.
WARC_DIR="./servo-warc-tests"
WARC_REPO="https://github.com/servo/servo-warc-tests.git"

# Clone the warc tests if they don't exist
if [[ ! -d ${WARC_DIR} ]]; then
    git clone --progress ${WARC_REPO}
fi

# Make sure we're running with an up-to-date warc test repo
git -C ${WARC_DIR} pull --progress

virtualenv venv --python="$(which python3)"
PS1="" source venv/bin/activate
# `PS1` must be defined before activating virtualenv
pip install \
    "boto3>=1.4.0" \
    git+https://github.com/ikreymer/pywb.git

mkdir -p servo
mkdir -p output # Test result will be saved to output/perf-<timestamp>.json
./git_log_to_json.sh > servo/revision.json

./test_all.sh --servo ${*}
SERVO_DIR="../../.." ${WARC_DIR}/run-warc-tests.sh
