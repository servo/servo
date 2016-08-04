#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

set -o errexit
set -o nounset
set -o pipefail

TARGET_DIR="${OUT_DIR}/../../.."
arm-linux-androideabi-gcc "${@}" \
                          "${LDFLAGS-}" -lc -shared \
                          -o "${TARGET_DIR}/libservo.so"
touch "${TARGET_DIR}/servo"
