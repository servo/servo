#!/bin/bash

# Run in the tools directory.
cd "$(dirname $0)"

# Setup and build bindgen.
if [ "$(uname)" == "Linux" ]; then
  export LIBCLANG_PATH=/usr/lib/llvm-3.8/lib;
else
  export LIBCLANG_PATH=`brew --prefix llvm38`/lib/llvm-3.8/lib;
fi

# Make sure we have llvm38.
if [ ! -x "$(command -v clang++-3.8)" ]; then
    echo "llmv38 must be installed. Mac users should |brew install llvm38|, Linux varies by distro."
    exit 1
fi

export LD_LIBRARY_PATH=$LIBCLANG_PATH
export DYLD_LIBRARY_PATH=$LIBCLANG_PATH

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
