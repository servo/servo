#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

set -o errexit
set -o nounset
set -o pipefail

# Run in the tools directory.
cd "$(dirname ${0})"

# Setup and build bindgen.
if [[ "$(uname)" == "Linux" ]]; then
  export LIBCLANG_PATH=/usr/lib/llvm-3.8/lib
else
  export LIBCLANG_PATH="$(brew --prefix llvm38)/lib/llvm-3.8/lib"
fi

# Make sure we have llvm38.
if [[ ! -x "$(command -v clang-3.8)" ]]; then
    echo "llmv38 must be installed." \
         "Mac users should |brew install llvm38|, Linux varies by distro."
    exit 1
fi

export LD_LIBRARY_PATH="${LIBCLANG_PATH}"
export DYLD_LIBRARY_PATH="${LIBCLANG_PATH}"

# Check for multirust
if [[ ! -x "$(command -v multirust)" ]]; then
    echo "multirust must be installed."
    exit 1
fi

# Don't try to clone twice.
if [[ ! -d rust-bindgen ]]; then
  git clone https://github.com/servo/rust-bindgen.git
fi

cd rust-bindgen

multirust override nightly
cargo build --features llvm_stable
