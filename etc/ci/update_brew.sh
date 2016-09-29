#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

set -o errexit
set -o nounset
set -o pipefail

SCRIPTDIR=${PWD}/$(dirname ${0})
cd "${SCRIPTDIR}/../.."

PACKAGEPATH=$(ls -t target/brew/servo-????-??-??.tar.gz | head -n 1)
PACKAGENAME=$(basename ${PACKAGEPATH})
REGEX="s/servo-.*\([0-9]\{4\}\)-\([0-9]\{2\}\)-\([0-9]\{2\}\).tar.gz/\1.\2.\3/p"
VERSION=$(echo ${PACKAGENAME}| sed -n "${REGEX}")
SHA=$(shasum -a 256 ${PACKAGEPATH} | sed -e 's/ .*//')

# See upload_nightly.sh
PACKAGEURL="https://download.servo.org/nightly/macbrew/${PACKAGENAME}"

if [[ -z ${VERSION} ]]; then
  echo "Package doesn't havent the right format: ${PACKAGENAME}"
  exit 1
fi

TMP_DIR=$(mktemp -d -t homebrew-servo)
cd ${TMP_DIR}
echo ${TMP_DIR}

echo "Cloning"
git clone https://github.com/servo/homebrew-servo.git
cd homebrew-servo

# Not using "/" as it's used in PACKAGEURL
cat ${SCRIPTDIR}/servo-binary-formula.rb.in | sed \
  "s|PACKAGEURL|${PACKAGEURL}|g
   s|SHA|${SHA}|g
   s|VERSION|${VERSION}|g" > Formula/servo-bin.rb

git add ./Formula/servo-bin.rb
git commit -m "Version bump: ${VERSION}"

git push -qf \
    "https://${TOKEN}@github.com/servo/homebrew-servo.git" master \
    >/dev/null 2>&1
rm -rf ${TMP_DIR}
