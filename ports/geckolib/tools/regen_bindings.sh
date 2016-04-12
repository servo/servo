#!/bin/bash

# Run in the tools directory.
cd "$(dirname $0)"

if [ $# -ne 1 ]; then
  echo "Usage: $0 /path/to/gecko/objdir"
  exit 1
fi

# Check for rust-bindgen
if [ ! -d rust-bindgen ]; then
  echo "rust-bindgen not found. Run setup_bindgen.sh first."
  exit 1
fi

export RUST_BACKTRACE=1
export LIBCLANG_PATH="$(pwd)/llvm/build/Release+Asserts/lib"
export DYLD_LIBRARY_PATH="$(pwd)/llvm/build/Release+Asserts/lib"
export LD_LIBRARY_PATH="$(pwd)/llvm/build/Release+Asserts/lib"
export DIST_INCLUDE="$1/dist/include"

# Prevent bindgen from generating opaque types for the gecko style structs.
export MAP_GECKO_STRUCTS=""
for STRUCT in nsStyleFont nsStyleColor nsStyleList nsStyleText \
              nsStyleVisibility nsStyleUserInterface nsStyleTableBorder \
              nsStyleSVG nsStyleVariables nsStyleBackground nsStylePosition \
              nsStyleTextReset nsStyleDisplay nsStyleContent nsStyleUIReset \
              nsStyleTable nsStyleMargin nsStylePadding nsStyleBorder \
              nsStyleOutline nsStyleXUL nsStyleSVGReset nsStyleColumn nsStyleEffects
do
  MAP_GECKO_STRUCTS=$MAP_GECKO_STRUCTS"-blacklist-type $STRUCT "
  MAP_GECKO_STRUCTS=$MAP_GECKO_STRUCTS"-raw-line 'use gecko_style_structs::$STRUCT;'$'\n' "
done

# Check for the include directory.
if [ ! -d "$DIST_INCLUDE" ]; then
  echo "$DIST_INCLUDE: directory not found"
  exit 1
fi

# We need to use 'eval' here to make MAP_GECKO_STRUCTS evaluate properly as
# multiple arguments.
#
# Uncomment the following line to run rust-bindgen in a debugger on mac.
# The absolute path is required to allow launching lldb with an untrusted
# library in DYLD_LIBRARY_PATH.
#
# /Applications/Xcode.app/Contents/Developer/usr/bin/lldb --
eval ./rust-bindgen/target/debug/bindgen           \
  -x c++ -std=gnu++0x                              \
  "-I$DIST_INCLUDE"                                \
  -o ../bindings.rs                                \
  -no-type-renaming                                \
  "$DIST_INCLUDE/mozilla/ServoBindings.h"          \
  -match "ServoBindings.h"                         \
  -match "nsStyleStructList.h"                     \
  $MAP_GECKO_STRUCTS
