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

# Make sure we have llvm-3.8.
if [[ ! -x "$(command -v clang-3.8)" ]]; then
  echo "llvm-3.8 is required." \
       "Mac users should |brew install llvm38|," \
       "Linux users can find it in clang-3.8."
  exit 1
fi

export LD_LIBRARY_PATH="${LIBCLANG_PATH}"
export DYLD_LIBRARY_PATH="${LIBCLANG_PATH}"
