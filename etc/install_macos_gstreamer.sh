#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

set -o errexit
set -o nounset
set -o pipefail

VERSION=1.22.2
URL_BASE=https://github.com/servo/servo-build-deps/releases/download/macOS

cd /tmp
curl -L "${URL_BASE}/gstreamer-1.0-${VERSION}-universal.pkg" -o gstreamer.pkg
curl -L "${URL_BASE}/gstreamer-1.0-devel-${VERSION}-universal.pkg" \
    -o gstreamer-dev.pkg
sudo installer -pkg 'gstreamer.pkg' -target /
sudo installer -pkg 'gstreamer-dev.pkg' -target /
