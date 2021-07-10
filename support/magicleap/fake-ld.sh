#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

set -o errexit
set -o nounset
set -o pipefail

MAGICLEAP_TOOLCHAIN=${MAGICLEAP_TOOLCHAIN:-"${MAGICLEAP_SDK}/tools/toolchains"}
TARGET=${TARGET:-"aarch64-linux-android"}
LD=${LD:-"${MAGICLEAP_TOOLCHAIN}/bin/${TARGET}-ld"}
LDFLAGS=${LDFLAGS:-"-L${MAGICLEAP_SDK}/lumin/stl/libc++/lib -L${MAGICLEAP_SDK}/lumin/usr/lib -L${MAGICLEAP_TOOLCHAIN}/lib/gcc/${TARGET}/4.9.x"}

# Remove the -landroid flag, grr
ARGS=("$@")
ARGS=${ARGS[@]/-landroid}

echo ${LD} ${LDFLAGS} ${ARGS}
${LD} ${LDFLAGS} ${ARGS}
