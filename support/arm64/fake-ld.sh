#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

echo /usr/bin/aarch64-linux-gnu-ld -L/usr/lib/aarch64-linux-gnu $*
/usr/bin/aarch64-linux-gnu-ld -L/usr/lib/aarch64-linux-gnu $*
