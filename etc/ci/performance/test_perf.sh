#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

set -o errexit
set -o nounset
set -o pipefail

TP5N_SOURCE="https://people.mozilla.org/~jmaher/taloszips/zips/tp5n.zip"
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

virtualenv venv --python="$(which python3)"
PS1="" source venv/bin/activate
# `PS1` must be defined before activating virtualenv
pip install "treeherder-client>=3.0.0"

mkdir -p servo
mkdir -p output
./git_log_to_json.sh > servo/revision.json && \
./test_all.sh --servo
