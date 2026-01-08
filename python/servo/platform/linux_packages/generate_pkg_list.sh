#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

# This file converts the input files, which are line separated and may contain `#` prefixed comments
# to a space separated list, without comments, and prints the result to stdout.

set -o errexit
set -o nounset
set -o pipefail

# 1. Read all input files
# 2. Remove comments (lines beginning with #)
# 3. Transform to space separated list
# 4. Trim whitespace
PACKAGE_LIST=$(cat "${@}" | grep -v '^#' | tr '\n' ' ' | awk '{$1=$1;print}')

echo "${PACKAGE_LIST}"
