#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/. */

# This file downloads and installs meson which is required for building
# osmesa-src, a dependency of wrench.

set -o errexit
set -o nounset
set -o pipefail
set -o xtrace

MESON_VER=0.55.1
MESON_BASE_URL="https://github.com/mesonbuild/meson/releases/download"

curl -L ${MESON_BASE_URL}/${MESON_VER}/meson-${MESON_VER}.tar.gz -o meson.tar.gz
tar -xf meson.tar.gz
mv meson-${MESON_VER} meson
cd meson
ln -s meson.py meson
