#!/bin/bash
# Cross-compile environment for Android on ARMv7 and x86
#
# Contents licensed under the terms of the OpenSSL license
# http://www.openssl.org/source/license.html
#
# See http://wiki.openssl.org/index.php/FIPS_Library_and_Android
#   and http://wiki.openssl.org/index.php/Android

#####################################################################

# Set ANDROID_NDK_ROOT to you NDK location. For example,
# /opt/android-ndk-r8e or /opt/android-ndk-r9. This can be done in a
# login script. If ANDROID_NDK_ROOT is not specified, the script will
# try to pick it up with the value of _ANDROID_NDK_ROOT below. If
# ANDROID_NDK_ROOT is set, then the value is ignored.
# _ANDROID_NDK="android-ndk-r8e"
_ANDROID_NDK="android-ndk-r9"
# _ANDROID_NDK="android-ndk-r10"

# Set _ANDROID_EABI to the EABI you want to use. You can find the
# list in $ANDROID_NDK_ROOT/toolchains. This value is always used.
# _ANDROID_EABI="x86-4.6"
# _ANDROID_EABI="arm-linux-androideabi-4.6"

# Set _ANDROID_ARCH to the architecture you are building for.
# This value is always used.
# _ANDROID_ARCH=arch-x86


case $RUST_TARGET in
    armv7*)
      _ANDROID_TARGET="arm-linux-androideabi"
      _ANDROID_ARCH=arch-arm
      _OPENSSL_MACHINE="armv7"
      _OPENSSL_ARCH="arm"
      _OPENSSL_CONFIG="android-armv7"
      ;;
    arm*)
      _ANDROID_TARGET=$RUST_TARGET
      _ANDROID_ARCH=arch-arm
      _OPENSSL_MACHINE="arm"
      _OPENSSL_ARCH="arm"
      _OPENSSL_CONFIG="android-armv7"
      ;;
    aarch64*)
      _ANDROID_TARGET=$RUST_TARGET
      _ANDROID_ARCH=arch-arm64
      _OPENSSL_MACHINE="armv7"
      _OPENSSL_ARCH="arm64"
      _OPENSSL_CONFIG="linux-generic64 -DB_ENDIAN"
      ;;
    x86*)
      _ANDROID_TARGET=$RUST_TARGET
      _ANDROID_ARCH=arch-x86
      _OPENSSL_MACHINE="x86"
      _OPENSSL_ARCH="arm"
      _OPENSSL_CONFIG="android-x86"
      ;;
    *)
      echo "Error: Invalid TARGET platform: $RUST_TARGET"
      ;;
esac

_ANDROID_EABI="$_ANDROID_TARGET-4.9"


# Set _ANDROID_API to the API you want to use. You should set it
# to one of: android-14, android-9, android-8, android-14, android-5
# android-4, or android-3. You can't set it to the latest (for
# example, API-17) because the NDK does not supply the platform. At
# Android 5.0, there will likely be another platform added (android-22?).
# This value is always used.
# _ANDROID_API="android-14"
_ANDROID_API="$ANDROID_PLATFORM"
# _ANDROID_API="android-19"

#####################################################################

# If the user did not specify the NDK location, try and pick it up.
# We expect something like ANDROID_NDK_ROOT=/opt/android-ndk-r8e
# or ANDROID_NDK_ROOT=/usr/local/android-ndk-r8e.
export ANDROID_NDK_ROOT=$1
echo $ANDROID_NDK_ROOT
# Error checking
# ANDROID_NDK_ROOT should always be set by the user (even when not running this script)
# http://groups.google.com/group/android-ndk/browse_thread/thread/a998e139aca71d77
if [ -z "$ANDROID_NDK_ROOT" ] || [ ! -d "$ANDROID_NDK_ROOT" ]; then
  echo "Error: ANDROID_NDK_ROOT is not a valid path. Please edit this script."
  # echo "$ANDROID_NDK_ROOT"
  # exit 1
fi

# Error checking
if [ ! -d "$ANDROID_NDK_ROOT/toolchains" ]; then
  echo "Error: ANDROID_NDK_ROOT/toolchains is not a valid path. Please edit this script."
  # echo "$ANDROID_NDK_ROOT/toolchains"
  # exit 1
fi

# Error checking
if [ ! -d "$ANDROID_NDK_ROOT/toolchains/$_ANDROID_EABI" ]; then
  echo "Error: ANDROID_EABI is not a valid path. Please edit this script."
  # echo "$ANDROID_NDK_ROOT/toolchains/$_ANDROID_EABI"
  # exit 1
fi

#####################################################################

# Based on ANDROID_NDK_ROOT, try and pick up the required toolchain. We expect something like:
# /opt/android-ndk-r83/toolchains/arm-linux-androideabi-4.7/prebuilt/linux-x86_64/bin
# Once we locate the toolchain, we add it to the PATH. Note: this is the 'hard way' of
# doing things according to the NDK documentation for Ice Cream Sandwich.
# https://android.googlesource.com/platform/ndk/+/ics-mr0/docs/STANDALONE-TOOLCHAIN.html

ANDROID_TOOLCHAIN=""
for host in "linux-x86_64" "linux-x86" "darwin-x86_64" "darwin-x86"
do
  if [ -d "$ANDROID_NDK_ROOT/toolchains/$_ANDROID_EABI/prebuilt/$host/bin" ]; then
    ANDROID_TOOLCHAIN="$ANDROID_NDK_ROOT/toolchains/$_ANDROID_EABI/prebuilt/$host/bin"
    break
  fi
done

# Error checking
if [ -z "$ANDROID_TOOLCHAIN" ] || [ ! -d "$ANDROID_TOOLCHAIN" ]; then
  echo "Error: ANDROID_TOOLCHAIN is not valid. Please edit this script."
  # echo "$ANDROID_TOOLCHAIN"
  # exit 1
fi

ANDROID_TOOLS="$_ANDROID_TARGET-gcc $_ANDROID_TARGET-ranlib $_ANDROID_TARGET-ld"

for tool in $ANDROID_TOOLS
do
  # Error checking
  if [ ! -e "$ANDROID_TOOLCHAIN/$tool" ]; then
    echo "Error: Failed to find $tool. Please edit this script."
    # echo "$ANDROID_TOOLCHAIN/$tool"
    # exit 1
  fi
done

# Only modify/export PATH if ANDROID_TOOLCHAIN good
if [ ! -z "$ANDROID_TOOLCHAIN" ]; then
  export ANDROID_TOOLCHAIN="$ANDROID_TOOLCHAIN"
  export PATH="$ANDROID_TOOLCHAIN":"$PATH"
fi

#####################################################################

# For the Android SYSROOT. Can be used on the command line with --sysroot
# https://android.googlesource.com/platform/ndk/+/ics-mr0/docs/STANDALONE-TOOLCHAIN.html
export ANDROID_SYSROOT="$ANDROID_NDK_ROOT/platforms/$_ANDROID_API/$_ANDROID_ARCH"
export SYSROOT="$ANDROID_SYSROOT"
export NDK_SYSROOT="$ANDROID_SYSROOT"

# Error checking
if [ -z "$ANDROID_SYSROOT" ] || [ ! -d "$ANDROID_SYSROOT" ]; then
  echo "Error: ANDROID_SYSROOT is not valid. Please edit this script."
  # echo "$ANDROID_SYSROOT"
  # exit 1
fi

#####################################################################

#####################################################################

# Most of these should be OK (MACHINE, SYSTEM, ARCH). RELEASE is ignored.
export MACHINE=$_OPENSSL_MACHINE
export RELEASE=2.6.37
export SYSTEM=android
export ARCH=$_OPENSSL_ARCH
export CROSS_COMPILE="$_ANDROID_TARGET-"

# For the Android toolchain
# https://android.googlesource.com/platform/ndk/+/ics-mr0/docs/STANDALONE-TOOLCHAIN.html
export ANDROID_SYSROOT="$ANDROID_NDK_ROOT/platforms/$_ANDROID_API/$_ANDROID_ARCH"
export SYSROOT="$ANDROID_SYSROOT"
export CROSS_SYSROOT="$ANDROID_SYSROOT"
export NDK_SYSROOT="$ANDROID_SYSROOT"
export ANDROID_NDK_SYSROOT="$ANDROID_SYSROOT"
export ANDROID_API="$_ANDROID_API"

# CROSS_COMPILE and ANDROID_DEV are DFW (Don't Fiddle With). Its used by OpenSSL build system.
# export CROSS_COMPILE="arm-linux-androideabi-"
export ANDROID_DEV="$ANDROID_NDK_ROOT/platforms/$_ANDROID_API/$_ANDROID_ARCH/usr"
export HOSTCC=gcc

# See https://github.com/cocochpie/android-openssl/blob/master/build-all-arch.sh
xCFLAGS="-DSHARED_EXTENSION=.so -fPIC -DOPENSSL_PIC -DDSO_DLFCN -DHAVE_DLFCN_H -mandroid -I$ANDROID_DEV/include -B$ANDROID_DEV/lib -O3 -fomit-frame-pointer -Wall"

VERBOSE=1
if [ ! -z "$VERBOSE" ] && [ "$VERBOSE" != "0" ]; then
  echo "ANDROID_NDK_ROOT: $ANDROID_NDK_ROOT"
  echo "ANDROID_ARCH: $_ANDROID_ARCH"
  echo "ANDROID_EABI: $_ANDROID_EABI"
  echo "ANDROID_API: $ANDROID_API"
  echo "ANDROID_SYSROOT: $ANDROID_SYSROOT"
  echo "ANDROID_TOOLCHAIN: $ANDROID_TOOLCHAIN"
  echo "FIPS_SIG: $FIPS_SIG"
  echo "CROSS_COMPILE: $CROSS_COMPILE"
  echo "ANDROID_DEV: $ANDROID_DEV"
fi

cd openssl-$2
perl -pi -e 's/install: all install_docs install_sw/install: install_docs install_sw/g' Makefile.org

# The code being built isn't maintained by us, so we redirect stderr to stdout
# so that the warnings don't clutter up buildbot
./Configure shared -no-ssl2 -no-ssl3 -no-comp -no-hw -no-engine --openssldir=/usr/local/ssl/$ANDROID_API $_OPENSSL_CONFIG $xCFLAGS 2>&1
make depend 2>&1
make all 2>&1
