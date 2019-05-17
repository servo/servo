#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

set -o errexit
set -o nounset
set -o pipefail

FORMULAS=(gst-plugins-bad.rb)
for i in ${FORMULAS[@]}; do
    curl -o "${i}" "https://raw.githubusercontent.com/Homebrew/homebrew-core/master/Formula/${i}"
    patch -i "${i/.rb/.diff}"
done
