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

CARGOFLAGS=${CARGOFLAGS:-"--verbose"}  # default to --verbose if not set

pushd webrender
cargo build ${CARGOFLAGS} --no-default-features
cargo build ${CARGOFLAGS} --no-default-features --features capture
cargo build ${CARGOFLAGS} --features capture,profiler
cargo build ${CARGOFLAGS} --features replay
popd

pushd wrench
cargo build ${CARGOFLAGS} --features env_logger
OPTIMIZED=0 python script/headless.py reftest
popd

pushd examples
cargo build ${CARGOFLAGS}
popd

cargo test ${CARGOFLAGS} \
    --all --exclude compositor --exclude compositor-wayland \
    --exclude compositor-windows --exclude glsl-to-cxx --exclude swgl
