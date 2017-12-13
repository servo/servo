#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

set -x
set -o errexit
set -o nounset
set -o pipefail

./mach test-tidy --no-progress --all
./mach test-tidy --no-progress --self-test
env SERVO_RUSTC_LLVM_ASSERTIONS=1 ./mach build --dev
env SERVO_RUSTC_LLVM_ASSERTIONS=1 ./mach test-compiletest
env SERVO_RUSTC_LLVM_ASSERTIONS=1 ./mach test-unit
env SERVO_RUSTC_LLVM_ASSERTIONS=1 ./mach package --dev
env SERVO_RUSTC_LLVM_ASSERTIONS=1 ./mach build-cef
./mach build-geckolib
./mach test-stylo
bash ./etc/ci/lockfile_changed.sh
bash ./etc/ci/manifest_changed.sh
bash ./etc/ci/check_no_panic.sh
