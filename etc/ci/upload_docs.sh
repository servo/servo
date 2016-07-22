#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

# Helper script to upload docs to doc.servo.org.
# Requires ghp-import (from pip)
# GitHub API token must be passed in environment var TOKEN

set -o errexit
set -o nounset
set -o pipefail

cd "$(dirname $0)/../.."

./mach doc
# etc/doc.servo.org/index.html overwrites $(mach rust-root)/doc/index.html
cp etc/doc.servo.org/* target/doc/

python components/style/properties/build.py servo html

OUT_DIR="`pwd`/target/doc/servo" \
  make -f makefile.cargo -C components/script dom_docs
rm -rf target/doc/servo/.cache

ghp-import -n target/doc
git push -qf "https://${TOKEN}@github.com/servo/doc.servo.org.git" gh-pages
