#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/. */

# This must be run from the root webrender directory!
# Users may set the CARGOFLAGS environment variable to pass
# additional flags to cargo if desired.

set -o errexit
set -o nounset
set -o pipefail
set -o xtrace

CARGOFLAGS=${CARGOFLAGS:-""}  # default to empty if not set

pushd wrench
# Test that all shaders compile successfully.
python script/headless.py --precache test_init
python script/headless.py --precache --use-unoptimized-shaders test_init

python script/headless.py reftest
python script/headless.py rawtest
python script/headless.py test_invalidation
CXX=clang++ cargo run ${CARGOFLAGS} --release --features=software -- \
  --software --headless reftest
popd
