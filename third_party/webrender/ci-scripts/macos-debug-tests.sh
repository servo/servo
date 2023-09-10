#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/. */

# This must be run from the root webrender directory!
# Users may set the CARGOFLAGS environment variable to pass
# additional flags to cargo if desired.

# Note that this script is run in a special cross-compiling configuration,
# where CARGOTESTFLAGS includes `--no-run`, and the binaries produced by
# `cargo test` are run on a different machine. When making changes to this
# file, please ensure any such binaries produced by `cargo test` are not
# deleted, or they may not get run as expected.

set -o errexit
set -o nounset
set -o pipefail
set -o xtrace

CARGOFLAGS=${CARGOFLAGS:-"--verbose"}  # default to --verbose if not set
CARGOTESTFLAGS=${CARGOTESTFLAGS:-""}

pushd webrender
cargo check ${CARGOFLAGS} --no-default-features
cargo check ${CARGOFLAGS} --no-default-features --features capture
cargo check ${CARGOFLAGS} --features capture,profiler
cargo check ${CARGOFLAGS} --features replay
popd

pushd wrench
cargo check ${CARGOFLAGS} --features env_logger
popd

pushd examples
cargo check ${CARGOFLAGS}
popd

cargo test ${CARGOFLAGS} ${CARGOTESTFLAGS} \
    --all --exclude compositor --exclude compositor-wayland \
    --exclude compositor-windows --exclude glsl-to-cxx --exclude swgl
