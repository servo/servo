#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

set -o errexit
set -o nounset
set -o pipefail

source fake-ld.sh

export _GCC_PARAMS="${@}"
call_gcc "arch-arm" "arm-linux-androideabi" "android-18" "armeabi-v7a"
