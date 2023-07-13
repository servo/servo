#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/. */

# This must be run from the root webrender directory!
# Users may set the CARGOFLAGS environment variable to pass
# additional flags to cargo if desired.
# The WRENCH_BINARY environment variable, if set, is used to run
# the precached reftest.

set -o errexit
set -o nounset
set -o pipefail
set -o xtrace

CARGOFLAGS=${CARGOFLAGS:-""}  # default to empty if not set
WRENCH_BINARY=${WRENCH_BINARY:-""}

pushd wrench

# Test that all shaders compile successfully.
python script/headless.py --precache test_init
python script/headless.py --precache --use-unoptimized-shaders test_init

python script/headless.py reftest
python script/headless.py test_invalidation
if [[ -z "${WRENCH_BINARY}" ]]; then
    cargo build ${CARGOFLAGS} --release
    WRENCH_BINARY="../target/release/wrench"
fi
"${WRENCH_BINARY}" --precache \
    reftest reftests/clip/fixed-position-clipping.yaml
popd
