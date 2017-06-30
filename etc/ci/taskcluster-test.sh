#! /bin/bash

./mach clean-nightlies --keep 3 --force
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
