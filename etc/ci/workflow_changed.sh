#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

set -o errexit
set -o nounset
set -o pipefail

python3 etc/ci/generate_workflow.py

diff="$(find . -name 'main.yml' -print0 | xargs -0 git diff)"
echo "${diff}"
[[ -z "${diff}" ]]
