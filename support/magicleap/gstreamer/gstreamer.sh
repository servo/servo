#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

set -o errexit
set -o nounset
set -o pipefail

SOURCE_DIR=gst-build
BUILD_DIR=_build
INSTALL_DIR=_install
INSTALL_REAL_DIR=$(realpath ${INSTALL_DIR})
ARCHIVE=gstreamer-magicleap-1.16.0-$(date +"%Y%m%d-%H%M%S").tgz

function build_and_install()
{
  ninja -C ${BUILD_DIR}
  DESTDIR=${INSTALL_REAL_DIR} meson install -C ${BUILD_DIR} --only-changed
  echo Creating archive ${ARCHIVE}.
  tar czf ${ARCHIVE} -C ${INSTALL_DIR} system
}

if [[ "${1:-}" == "--build-only" ]]; then
  build_and_install
  exit
fi

rm -rf ${BUILD_DIR}
rm -rf ${INSTALL_DIR}

# FIXME: Download, build and install GNU libiconv because MLSDK has an old
# version of bionic that does not include iconv.
ICONV_NAME=libiconv-1.16
if [[ ! -d ${ICONV_NAME} ]]; then
  curl -O -L https://ftp.gnu.org/pub/gnu/libiconv/${ICONV_NAME}.tar.gz
  tar xzf ${ICONV_NAME}.tar.gz
fi
mkdir -p ${BUILD_DIR}/${ICONV_NAME}
HOST=aarch64-linux-android
SYSROOT=${MAGICLEAP_SDK}/lumin/usr

cd ${BUILD_DIR}/${ICONV_NAME}
env CFLAGS=--sysroot=${SYSROOT} \
    CPPFLAGS=--sysroot=${SYSROOT} \
    CC=${MAGICLEAP_SDK}/tools/toolchains/bin/${HOST}-clang \
    AR=${MAGICLEAP_SDK}/tools/toolchains/bin/${HOST}-ar \
    RANLIB=${MAGICLEAP_SDK}/tools/toolchains/bin/${HOST}-ranlib \
    ../../${ICONV_NAME}/configure --host=${HOST} \
                --with-sysroot=${SYSROOT} \
                --prefix /system \
                --libdir /system/lib64
cd ../..
make -C ${BUILD_DIR}/${ICONV_NAME}
DESTDIR=${INSTALL_REAL_DIR} make -C ${BUILD_DIR}/${ICONV_NAME} install

# Clone custom repo/branch of gst-build
if [[ ! -d ${SOURCE_DIR} ]]; then
  git clone https://gitlab.freedesktop.org/xclaesse/gst-build.git --branch magicleap ${SOURCE_DIR}
fi

# Generate cross file by replacing the MLSDK location
cat mlsdk.txt.in | sed s#@MAGICLEAP_SDK@#${MAGICLEAP_SDK}# \
                 | sed s#@INSTALL_DIR@#${INSTALL_REAL_DIR}# > mlsdk.txt

meson --cross-file mlsdk.txt \
      --prefix /system \
      --libdir lib64 \
      --libexecdir bin \
      -Db_pie=true \
      -Dcpp_std=c++11 \
      -Dpython=disabled \
      -Dlibav=disabled \
      -Ddevtools=disabled \
      -Dges=disabled \
      -Drtsp_server=disabled \
      -Domx=disabled \
      -Dvaapi=disabled \
      -Dsharp=disabled \
      -Dexamples=disabled \
      -Dgtk_doc=disabled \
      -Dintrospection=disabled \
      -Dnls=disabled \
      -Dbad=enabled \
      -Dgst-plugins-base:gl=enabled \
      -Dgst-plugins-base:gl_platform=egl \
      -Dgst-plugins-base:gl_winsys=android \
      -Dgst-plugins-good:soup=enabled \
      -Dgst-plugins-bad:gl=enabled \
      -Dgst-plugins-bad:magicleap=enabled \
      -Dgst-plugins-bad:dash=enabled \
      -Dglib:iconv=gnu \
      -Dlibsoup:gssapi=false \
      -Dlibsoup:tls_check=false \
      -Dlibsoup:vapi=false \
      ${BUILD_DIR} \
      ${SOURCE_DIR}

build_and_install
