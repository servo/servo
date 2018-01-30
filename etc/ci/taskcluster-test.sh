#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

set -o errexit
set -o nounset
set -o pipefail

# Update this from the linux-dev builder in etc/ci/buildbot_steps.yml
./mach test-tidy --no-progress --all
./mach test-tidy --no-progress --self-test
env CC=gcc-5 CXX=g++-5 ./mach build --dev
env ./mach test-unit
env ./mach package --dev
env ./mach build-cef
env ./mach build --dev --no-default-features --features default-except-unstable
./mach build-geckolib
./mach test-stylo
bash ./etc/ci/lockfile_changed.sh
bash ./etc/ci/manifest_changed.sh
bash ./etc/ci/check_no_panic.sh
