#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

set -o errexit
set -o nounset
set -o pipefail

NDK=../../../android-toolchains/android-ndk-r12b-linux-x86_64/android-ndk-r12b
BIN="${NDK}/toolchains/x86-4.9/prebuilt/linux-x86_64/bin/"

"${BIN}/i686-linux-android-gcc" \
  --sysroot "${NDK}/platforms/android-18/arch-x86" \
  "${@}"
