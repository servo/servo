#!/usr/bin/env bash
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

set -ex
set -o errexit
set -o nounset
set -o pipefail

if test ! -e "rust-toolchain.toml"; then
  echo "Please run script in servo git repository"
  exit
fi

echo "Syncing Rust dependencies..."
git clone https://github.com/flatpak/flatpak-builder-tools.git
{
  pushd flatpak-builder-tools
  pushd cargo
  python3 ./flatpak-cargo-generator.py ../../Cargo.lock \
    -t -o ../../support/flatpak/cargo-sources.json
  popd

  pushd pip
  # hack for missing setuptools_rust
  {
    python3 ./flatpak-pip-generator flit_core \
      -o ../../support/flatpak/python3-flit_core
    python3 ./flatpak-pip-generator setuptools_rust \
      -o ../../support/flatpak/python3-setuptools_rust
  }
  # other manually listed jsons FIXME remove in future!
  {
    # FIXME hardcoded
    #python3 ./flatpak-pip-generator cryptography \
    #  -o ../../support/flatpak/python3-cryptography

    python3 ./flatpak-pip-generator pathspec \
      -o ../../support/flatpak/python3-pathspec
    python3 ./flatpak-pip-generator pluggy \
      -o ../../support/flatpak/python3-pluggy
    python3 ./flatpak-pip-generator tomli \
      -o ../../support/flatpak/python3-tomli
    python3 ./flatpak-pip-generator trove_classifiers \
      -o ../../support/flatpak/python3-trove_classifiers
    python3 ./flatpak-pip-generator packaging \
      -o ../../support/flatpak/python3-packaging
    python3 ./flatpak-pip-generator hatch-vcs \
      -o ../../support/flatpak/python3-hatch-vcs
  }


  # main dependencies
  sed '/dataclasses/d' ../../python/requirements.txt > req.txt
  python3 ./flatpak-pip-generator -r req.txt \
    -o ../../support/flatpak/python3-requirements
  rm req.txt

  popd
  pushd cargo
  # hack for cargo inside cryptography
  {
    DOWNLOAD_CRYPTOGRAPHY=$(grep "cryptography-" ../../support/flatpak/python3-requirements.json | cut -d'"' -f4)
    curl -L -o cryptography.tar.gz "${DOWNLOAD_CRYPTOGRAPHY}"

    tar -xzf cryptography.tar.gz
    python3 ./flatpak-cargo-generator.py cryptography-*/src/rust/Cargo.lock \
      -t -o ../../support/flatpak/python3-cargo-sources.json

  }
  popd

  popd
}
rm -rf flatpak-builder-tools

echo "Syncing Rush nightly version..."

VER_FULL=$(grep "channel =" rust-toolchain.toml | cut -d'"' -f2)
VER=${VER_FULL#nightly-}

rust_dist="https://static.rust-lang.org/dist/"

arr1="${rust_dist}${VER}/rust-nightly-aarch64-unknown-linux-gnu.tar.xz"
arr2="${rust_dist}${VER}/rust-nightly-x86_64-unknown-linux-gnu.tar.xz"
arr3="${rust_dist}${VER}/rustc-dev-nightly-aarch64-unknown-linux-gnu.tar.xz"
arr4="${rust_dist}${VER}/rustc-dev-nightly-x86_64-unknown-linux-gnu.tar.xz"

OUTPUT="support/flatpak/org.servo.Servo.json"

cp "support/flatpak/org.servo.Servo.template.json" ${OUTPUT}
i=1
for str in ${arr1} ${arr2} ${arr3} ${arr4}; do
  sed -i "s|_REPLACE_${i}_|${str}|g" ${OUTPUT}
  sha256=$(curl -L "${str}" | sha256sum | cut -f1 -d' ')
  sed -i "s|_REPLACE_SHA_${i}_|${sha256}|g" ${OUTPUT}
  ((i++))
done
