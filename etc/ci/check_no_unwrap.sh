#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

# Make sure listed files do not contain "unwrap"

set -o errexit
set -o nounset
set -o pipefail

# cd into repo root to make sure paths work in any case
cd "$(git rev-parse --show-toplevel)"

# files that should not contain "unwrap"
FILES=("components/compositing/compositor.rs"
       "components/constellation/constellation.rs"
       "components/constellation/pipeline.rs"
       "ports/glutin/lib.rs"
       "ports/glutin/window.rs")

# make sure the files exist
ls -1 "${FILES[@]}"

# make sure the files do not contain "unwrap" or "panic!"
! grep --line-number --with-filename "unwrap(\|panic!(" "${FILES[@]}"
