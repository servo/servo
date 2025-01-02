#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

set -o errexit
set -o nounset
set -o pipefail

PS1="" source python/_virtualenv/bin/activate
# `PS1` must be defined before activating virtualenv
python python/servo/mutation/init.py components/script/dom
