#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

set -o errexit
set -o nounset
set -o pipefail

# Any
target=""

# Specific device
#target="-s something"

# Emulator
#target="-e"

# USB
#target="-d"

path="${1}"
base="$(basename ${1})"
remote_path="/data/local/tmp/${base}"
shift

adb ${target} "wait-for-device"
adb ${target} push "${path}" "${remote_path}" >&2
adb ${target} shell "${remote_path}" "${@}"
