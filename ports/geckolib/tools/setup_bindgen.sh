#!/bin/bash

# Run in the tools directory.
cd "$(dirname $0)"

# Setup and build bindgen.
export LIBCLANG_PATH="$(pwd)/llvm/build/lib"
export LD_LIBRARY_PATH="$(pwd)/llvm/build/lib"
export DYLD_LIBRARY_PATH="$(pwd)/llvm/build/lib"


# Make sure we have a custom clang set up.
if [ ! -d "$LIBCLANG_PATH" ]; then
  echo "Custom LLVM/Clang not found. Run build_custom_clang.sh first."
  exit 1
fi

# Check for multirust
if [ ! -x "$(command -v multirust)" ]; then
    echo "multirust must be installed."
    exit 1
fi

# Don't try to clone twice.
if [ ! -d rust-bindgen ]; then
  git clone https://github.com/ecoal95/rust-bindgen.git
  cd rust-bindgen
  git checkout sm-hacks-rebase-squashed
else
  cd rust-bindgen
fi

multirust override nightly
cargo build
