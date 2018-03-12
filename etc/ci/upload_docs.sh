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

cd "$(dirname ${0})/../.."

# Clean up any traces of previous doc builds.
./etc/ci/clean_build_artifacts.sh

env CC=gcc-5 CXX=g++-5 ./mach doc
# etc/doc.servo.org/index.html overwrites $(mach rust-root)/doc/index.html
# Use recursive copy here to avoid `cp` returning an error code
# when it encounters directories.
cp -r etc/doc.servo.org/* target/doc/

./mach cargo-geckolib doc
# Use recursive copy here to avoid `cp` returning an error code
# when it encounters directories.
cp -r target/geckolib/doc/* target/doc/geckolib/

python components/style/properties/build.py servo html regular

cd components/script
cmake .
cmake --build . --target supported-apis
echo "Copying apis.html."
cp apis.html ../../target/doc/servo/
echo "Copied apis.html."
cd ../..

echo "Starting ghp-import."
ghp-import -n target/doc
echo "Finished ghp-import."
git push -qf \
    "https://${TOKEN}@github.com/servo/doc.servo.org.git" gh-pages \
    &>/dev/null
echo "Finished git push."

# Clean up the traces of the current doc build.
./etc/ci/clean_build_artifacts.sh
