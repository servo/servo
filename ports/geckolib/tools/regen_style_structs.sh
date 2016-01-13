#!/bin/bash

# Run in the tools directory.
cd `dirname $0`

if [ $# -ne 1 ]; then
  echo "Usage: $0 /path/to/objdir/"
  exit 1
fi

# Check for rust-bindgen
if [ ! -d rust-bindgen ]; then
  echo "rust-bindgen not found. Run setup_bindgen.sh first."
  exit 1
fi

# Need to find a way to avoid hardcoding these
STD_LIB_PATH=/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX10.10.sdk/usr/include
STDXX_LIB_PATH=/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/include/c++/v1
if [ ! -d $STD_LIB_PATH ] || [ ! -d $STDXX_LIB_PATH ]; then
  echo "Please update the above environmental variables to point to your standard library."
  exit 1
fi

export RUST_BACKTRACE=1
export LIBCLANG_PATH=`pwd`/llvm/build/Release+Asserts/lib
export DYLD_LIBRARY_PATH=`pwd`/llvm/build/Release+Asserts/lib
export DIST_INCLUDE=$1/dist/include

# Check for the include directory.
if [ ! -d $DIST_INCLUDE ]; then
  echo "$DIST_INCLUDE: directory not found"
  exit 1
fi

# Uncomment the following line to run rust-bindgen in a debugger on mac.
# The absolute path is required to allow launching lldb with an untrusted
# library in DYLD_LIBRARY_PATH.
#
# /Applications/Xcode.app/Contents/Developer/usr/bin/lldb --
./rust-bindgen/target/debug/bindgen -x c++ -std=gnu++0x -ignore-functions -allow-unknown-types -isystem $STDXX_LIB_PATH -isystem $STD_LIB_PATH -I$DIST_INCLUDE -I$DIST_INCLUDE/nspr -DDEBUG=1 -DTRACING=1 -DOS_POSIX=1 -DOS_MACOSX=1 -DMOZILLA_INTERNAL_API -DIMPL_LIBXUL  -include $1/mozilla-config.h -o ../gecko_style_structs.rs $DIST_INCLUDE/nsStyleStruct.h
