#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

set -o errexit
set -o nounset
set -o pipefail

if [[ -z "${OPENSSL_DIR}" ]]; then
  echo "No OPENSSL_DIR."
  exit 1
fi

if [[ -z "${OPENSSL_VERSION}" ]]; then
  echo "No OPENSSL_VERSION."
  exit 1
fi

if [[ -f "${OPENSSL_DIR}/lib/libssl.so" ]] && \
   [[ "${OPENSSL_DIR}/lib/libssl.so" -nt "${0}" ]] ; then
  exit 0
fi

echo "Building ${OPENSSL_DIR}/lib/libssl.so"

S3_BUCKET="https://servo-deps-2.s3.amazonaws.com/android-deps"
S3_URL="${S3_BUCKET}/openssl-${OPENSSL_VERSION}.tar.gz"

if [[ ! -d "${OPENSSL_DIR}/src/openssl-${OPENSSL_VERSION}" ]]; then
  mkdir -p "${OPENSSL_DIR}/src"
  curl "${S3_URL}" | tar xzf - -C "${OPENSSL_DIR}/src"
fi

if [[ ! -d "${OPENSSL_DIR}/src/openssl-${OPENSSL_VERSION}" ]]; then
  echo "Failed to download ${OPENSSL_DIR}/src/openssl-${OPENSSL_VERSION}"
  exit 1
fi

cd "${OPENSSL_DIR}/src/openssl-${OPENSSL_VERSION}"

./Configure shared \
  --prefix="${OPENSSL_DIR}" \
  --openssldir="${OPENSSL_DIR}" \
  -no-ssl2 -no-ssl3 -no-comp -no-engine -no-hw \
  linux-generic64 \
  -fPIC -fno-omit-frame-pointer \
  -Wall -Wno-error=macro-redefined -Wno-unknown-attributes \
  ${CFLAGS:-}

make depend
make all
make install_sw

if [[ ! -f "${OPENSSL_DIR}/lib/libssl.so" ]]; then
  echo "Failed to build ${OPENSSL_DIR}/lib/libssl.so"
  exit 1
fi

