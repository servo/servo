#!/usr/bin/env bash

./mach build --rel --target=arm-unknown-linux-gnueabihf || cat target/arm-unknown-linux-gnueabihf/release/build/backtrace-sys-*/out/config.*
