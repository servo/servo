#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

set -o errexit
set -o nounset
set -o pipefail

set -x

curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain none -y
export PATH="${HOME}/.cargo/bin:${PATH}"

# Update this from the linux-dev builder in etc/ci/buildbot_steps.yml

export RUST_BACKTRACE=1
export RUSTFLAGS="-Dwarnings"
export CARGO_INCREMENTAL=0
export SCCACHE_IDLE_TIMEOUT=1200

./mach test-tidy --no-progress --all
./mach test-tidy --no-progress --self-test
env CC=gcc-5 CXX=g++-5 ./mach build --dev
env ./mach test-unit
env ./mach package --dev
env ./mach build --dev --no-default-features --features default-except-unstable
bash ./etc/ci/lockfile_changed.sh
bash ./etc/ci/check_no_panic.sh
