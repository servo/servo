#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

set -o errexit
set -o nounset
set -o pipefail

# If we somehow ended up in an unclean state previously, attempt
# to set up a clean environment for testing.
hdiutil detach /Volumes/Servo >/dev/null 2>&1 || true;

# Mount the package that will be tested.
hdiutil attach ${1}
pushd /Volumes/Servo/Servo.app/Contents/MacOS
ls -l

# Load a page that closes immediately after loading.
c='data:text/html,<script>onload=()=>{console.log("success");close()}</script>'
./servo --headless ${c} | tee /tmp/out
grep 'success' /tmp/out

# Clean up.
popd

hdiutil detach /Volumes/Servo || \
    echo "WARNING: Could not detach /Volumes/Servo. " \
         "Please detach with hdiutil manually."
