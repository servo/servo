#!/usr/bin/env bash

./mach build --rel --target=arm-unknown-linux-gnueabihf || cat /home/servo/buildbot/slave/arm32/build/target/arm-unknown-linux-gnueabihf/release/build/backtrace-sys-*/out/config.*
