#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

echo /usr/bin/arm-linux-gnueabihf-ld -L/usr/lib/arm-linux-gnueabihf $*
/usr/bin/arm-linux-gnueabihf-ld -L/usr/lib/arm-linux-gnueabihf $*
