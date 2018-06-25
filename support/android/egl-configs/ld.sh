#!/bin/sh
NDK=../../../android-toolchains/android-ndk-r12b-linux-x86_64/android-ndk-r12b

"${NDK}/toolchains/arm-linux-androideabi-4.9/prebuilt/linux-x86_64/bin/arm-linux-androideabi-gcc" \
  --sysroot "${NDK}/platforms/android-18/arch-arm" \
  "$@"
