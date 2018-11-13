#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

set -o errexit
set -o nounset
set -o pipefail

# We enable pipefail above to satisfy servo-tidy, but disable it again here.
# In the case of the 'yes' program,
# exiting when the stdout pipe is broken is expected.
set +o pipefail

cd $(dirname ${0})/../..
yes | ./mach bootstrap-android
