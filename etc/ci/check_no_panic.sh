#!/usr/bin/env bash
#
# Make sure listed files do not contain "unwrap"

set -o errexit
set -o nounset
set -o pipefail

cd "$(git rev-parse --show-toplevel)" # cd into repo root so make sure paths works in any case

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
