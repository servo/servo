#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

set -o errexit
set -o nounset
set -o pipefail

task_id="${1}"
artifact="${2}"
shift 2
queue="https://queue.taskcluster.net/v1"
url="${queue}/task/${task_id}/artifacts/public/${artifact}"
echo "Fetching ${url}" >&2
curl \
    --retry 5 \
    --connect-timeout 10 \
    --location \
    --fail \
    "${url}" \
    "${@}"
