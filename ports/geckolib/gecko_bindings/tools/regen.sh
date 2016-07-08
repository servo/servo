#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

set -o errexit
set -o nounset
set -o pipefail

if [ $# -eq 0 ]; then
  echo "Usage: $0 /path/to/gecko/objdir [other-regen.py-flags]"
  exit 1
fi

# Check for rust-bindgen
if [ ! -d rust-bindgen ]; then
  echo "rust-bindgen not found. Run setup_bindgen.sh first."
  exit 1
fi

# Check for /usr/include
if [ ! -d /usr/include ]; then
  echo "/usr/include doesn't exist." \
       "Mac users may need to run xcode-select --install."
  exit 1
fi

if [ "$(uname)" == "Linux" ]; then
  LIBCLANG_PATH=/usr/lib/llvm-3.8/lib;
else
  LIBCLANG_PATH=`brew --prefix llvm38`/lib/llvm-3.8/lib;
fi

./regen.py --target all "$@"
