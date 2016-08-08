#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

set -o errexit
set -o nounset
set -o pipefail

# TP5 manifest uses `localhost`, but our local server probably don't use port 80
sed 's/localhost\/page_load_test\/tp5n/localhost:8000\/page_load_test\/tp5n/g' \
  ./page_load_test/tp5n/tp5o.manifest > ./page_load_test/tp5n/tp5o_8000.manifest

