#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

# Make sure listed paths do not use unwrap() or panic!()

set -o errexit
set -o nounset
set -o pipefail

# cd into repo root to make sure paths work in any case
cd "$(git rev-parse --show-toplevel)"

# Each path can be either a single file or a directory
PATHS=(
    "components/compositing/compositor.rs"
    "components/constellation/"
    "ports/glutin/lib.rs"
    "ports/glutin/window.rs"
)

# Make sure the paths exist
ls -1 "${PATHS[@]}"

# Make sure the files do not contain "unwrap" or "panic!"
! grep \
    --dereference-recursive \
    --line-number \
    --with-filename \
    "unwrap(\|panic!(" \
    "${PATHS[@]}"
