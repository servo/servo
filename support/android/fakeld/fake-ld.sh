#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

set -o errexit
set -o nounset
set -o pipefail

call_gcc()
{
  TARGET_DIR="${OUT_DIR}/../../.."

  export _ANDROID_ARCH=$1
  export _ANDROID_TARGET=$3
  export ANDROID_SYSROOT="${ANDROID_NDK}/platforms/${ANDROID_PLATFORM}/${_ANDROID_ARCH}"
  ANDROID_TOOLCHAIN=""
  for host in "linux-x86_64" "linux-x86" "darwin-x86_64" "darwin-x86"; do
    if [[ -d "${ANDROID_NDK}/toolchains/llvm/prebuilt/${host}/bin" ]]; then
      ANDROID_TOOLCHAIN="${ANDROID_NDK}/toolchains/llvm/prebuilt/${host}/bin"
      break
    fi
  done

  ANDROID_CPU_ARCH_DIR=$2
  ANDROID_CXX_LIBS="${ANDROID_NDK}/sources/cxx-stl/llvm-libc++/libs/${ANDROID_CPU_ARCH_DIR}"

  echo "toolchain: ${ANDROID_TOOLCHAIN}"
  echo "libs dir: ${ANDROID_CXX_LIBS}"
  echo "sysroot: ${ANDROID_SYSROOT}"
  echo "targetdir: ${TARGET_DIR}"

  "${ANDROID_TOOLCHAIN}/clang" \
      --sysroot="${ANDROID_SYSROOT}" \
      --gcc-toolchain="${GCC_TOOLCHAIN}" \
      --target="${_ANDROID_TARGET}" \
      -L "${ANDROID_CXX_LIBS}" ${_GCC_PARAMS} -lc++
}
