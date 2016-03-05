#!/bin/bash

# Run in the tools directory.
cd "$(dirname $0)"

# Setup and build bindgen.
export LIBCLANG_PATH="$(pwd)/llvm/build/Release+Asserts/lib"


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
  git clone https://github.com/bholley/rust-bindgen.git
  cd rust-bindgen
  git checkout sm-hacks
else
  cd rust-bindgen
fi

multirust override nightly
cargo build
