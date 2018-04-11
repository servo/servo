#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

set -o errexit
set -o nounset
set -o pipefail

cd "$(git rev-parse --show-toplevel)"

# NOTE(emilio): This assumes that HEAD is a merge commit, which is true for
# builbot stuff. Locally you probably want to use `-1`.
LAST_MERGE="$(git log --merges --format=%h -2 | tail -1)"

cargo install --force rustfmt-nightly
git diff $LAST_MERGE | rustfmt-format-diff -p 1

git add -u
git diff @
git diff-index --quiet HEAD
