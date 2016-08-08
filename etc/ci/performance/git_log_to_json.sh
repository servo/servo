#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

set -o errexit
set -o nounset
set -o pipefail

# Don't include body to prevent multiline and unescaped body string
git log -n 1 --pretty=format:'{%n  "commit": "%H",%n  "subject": "%s",%n
  "author": {%n    "name": "%aN",%n    "email": "%aE",%n
    "timestamp": "%at"%n  }%n  %n}'

