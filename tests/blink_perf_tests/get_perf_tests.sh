#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

# script to vendor blink perf tests into servo.
# At the time of writing we are mainly interested in the layout tests.

set -o errexit
set -o nounset
set -o pipefail

CHROMIUM_TAG=141.0.7354.1
curl -LO https://chromium.googlesource.com/chromium/src/+archive/refs/tags/${CHROMIUM_TAG}/third_party/blink/perf_tests.tar.gz
# Delete any old directory contents.
rm -rf perf_tests && mkdir perf_tests
tar -xf perf_tests.tar.gz -C perf_tests
# Download license
curl -L https://chromium.googlesource.com/chromium/src/+/refs/tags/141.0.7354.1/third_party/blink/LICENSE_FOR_ABOUT_CREDITS?format=TEXT \
  | base64 -d > LICENSE_FOR_ABOUT_CREDITS
git add LICENSE_FOR_ABOUT_CREDITS
