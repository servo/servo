#!/bin/bash
#
# Make sure listed files do not use unwrap() or panic!()
set -o errexit
set -o nounset
set -o pipefail

cd $(git rev-parse --show-toplevel) # cd into repo root so make sure paths works in any case

FILES=("components/compositing/compositor.rs"
       "components/compositing/pipeline.rs"
       "components/compositing/constellation.rs")

! grep -n "unwrap(\|panic!(" "${FILES[@]}"
