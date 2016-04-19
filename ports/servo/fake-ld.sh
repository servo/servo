#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

set -o errexit
set -o nounset
set -o pipefail

TARGET_DIR="${OUT_DIR}/../../.."

export _ANDROID_ARCH=arch-arm
export ANDROID_SYSROOT="${ANDROID_NDK}/platforms/android-18/${_ANDROID_ARCH}"
export _ANDROID_EABI=arm-linux-androideabi-4.9
ANDROID_TOOLCHAIN=""
for host in "linux-x86_64" "linux-x86" "darwin-x86_64" "darwin-x86"; do
  if [[ -d "${ANDROID_NDK}/toolchains/${_ANDROID_EABI}/prebuilt/${host}/bin" ]]; then
    ANDROID_TOOLCHAIN="${ANDROID_NDK}/toolchains/${_ANDROID_EABI}/prebuilt/${host}/bin"
    break
  fi
done

ANDROID_CPU_ARCH_DIR=armeabi
ANDROID_CXX_LIBS="${ANDROID_NDK}/sources/cxx-stl/llvm-libc++/libs/${ANDROID_CPU_ARCH_DIR}"

echo "toolchain: ${ANDROID_TOOLCHAIN}"
echo "libs dir: ${ANDROID_CXX_LIBS}"
echo "sysroot: ${ANDROID_SYSROOT}"

"${ANDROID_TOOLCHAIN}/arm-linux-androideabi-gcc" \
  --sysroot="${ANDROID_SYSROOT}" -L "${ANDROID_CXX_LIBS}" "${@}" -lc++ \
  -o "${TARGET_DIR}/libservo.so" -shared && touch "${TARGET_DIR}/servo"
