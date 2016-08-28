#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

set -o errexit
set -o nounset
set -o pipefail

wget http://people.mozilla.org/~jmaher/taloszips/zips/tp5n.zip \
     -O page_load_test/tp5n.zip && \
unzip -o page_load_test/tp5n.zip -d page_load_test

virtualenv venv --python=/usr/bin/python3
set +u
source venv/bin/activate
set -u
pip install "treeherder-client>=3.0.0"

mkdir -p servo
mkdir -p output
./git_log_to_json.sh > servo/revision.json && \
cp -r ../../../resources servo && \
./test_all.sh --servo
