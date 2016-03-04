#!/bin/bash

# Run in the tools directory.
cd `dirname $0`

# Make sure we have a custom clang set up.
if [ ! -d llvm ]; then
  echo "Custom LLVM/Clang not found. Run build_custom_clang.sh first."
  exit 1
fi

# Don't run twice.
if [ -d rust-bindgen ]; then
  echo "rust-bindgen directory already exists."
  exit 1
fi

# Check for multirust
if [ ! -x "$(command -v multirust)" ]; then
    echo 'multirust must be installed.'
    exit 1
fi

# Setup and build bindgen.
export LIBCLANG_PATH=`pwd`/llvm/build/Release+Asserts/lib
git clone https://github.com/bholley/rust-bindgen.git
cd rust-bindgen
git checkout sm-hacks
multirust override nightly
cargo build
