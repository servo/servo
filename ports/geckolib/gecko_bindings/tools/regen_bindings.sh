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

# Check for /usr/include
if [ ! -d /usr/include ]; then
  echo "/usr/include doesn't exist. Mac users may need to run xcode-select --install."
  exit 1
fi

if [ "$(uname)" == "Linux" ]; then
  PLATFORM_DEPENDENT_DEFINES+="-DOS_LINUX";
  LIBCLANG_PATH=/usr/lib/llvm-3.8/lib;
else
  PLATFORM_DEPENDENT_DEFINES+="-DOS_MACOSX";
  LIBCLANG_PATH=`brew --prefix llvm38`/lib/llvm-3.8/lib;
fi

# Prevent bindgen from generating opaque types for common gecko types.
export MAP_GECKO_TYPES=""

# Extra code we want to generate.
export EXTRA_CODE="-raw-line 'use heapsize::HeapSizeOf;' "

# Style structs.
for STRUCT in nsStyleFont nsStyleColor nsStyleList nsStyleText \
              nsStyleVisibility nsStyleUserInterface nsStyleTableBorder \
              nsStyleSVG nsStyleVariables nsStyleBackground nsStylePosition \
              nsStyleTextReset nsStyleDisplay nsStyleContent nsStyleUIReset \
              nsStyleTable nsStyleMargin nsStylePadding nsStyleBorder \
              nsStyleOutline nsStyleXUL nsStyleSVGReset nsStyleColumn nsStyleEffects \
              nsStyleImage nsStyleGradient nsStyleCoord nsStyleGradientStop
do
  MAP_GECKO_TYPES=$MAP_GECKO_TYPES"-blacklist-type $STRUCT "
  MAP_GECKO_TYPES=$MAP_GECKO_TYPES"-raw-line 'use structs::$STRUCT;' "
  EXTRA_CODE=$EXTRA_CODE"-raw-line 'unsafe impl Send for $STRUCT {}' "
  EXTRA_CODE=$EXTRA_CODE"-raw-line 'unsafe impl Sync for $STRUCT {}' "
  EXTRA_CODE=$EXTRA_CODE"-raw-line 'impl HeapSizeOf for $STRUCT { fn heap_size_of_children(&self) -> usize { 0 } }' "
done

# Other mapped types.
for TYPE in SheetParsingMode nsMainThreadPtrHandle nsMainThreadPtrHolder nscolor nsFont \
            FontFamilyList FontFamilyType nsIAtom
do
  MAP_GECKO_TYPES=$MAP_GECKO_TYPES"-blacklist-type $TYPE "
  MAP_GECKO_TYPES=$MAP_GECKO_TYPES"-raw-line 'use structs::$TYPE;' "
done



# Check for the include directory.
export OBJDIR="$1"
export SRCDIR="$1/.."  # Not necessarily true, but let's assume.
export DIST_INCLUDE="$1/dist/include"
if [ ! -d "$DIST_INCLUDE" ]; then
  echo "$DIST_INCLUDE: directory not found"
  exit 1
fi

export RUST_BACKTRACE=1

# We need to use 'eval' here to make MAP_GECKO_TYPES evaluate properly as
# multiple arguments.
eval ./rust-bindgen/target/debug/bindgen           \
  -x c++ -std=gnu++0x                              \
  "-I$DIST_INCLUDE"                                \
  "-I$DIST_INCLUDE/nspr/"                          \
  "-I$1/nsprpub/pr/include/"                       \
  $PLATFORM_DEPENDENT_DEFINES                      \
  -DMOZILLA_INTERNAL_API                           \
  -DMOZ_STYLO_BINDINGS=1                           \
  -DJS_DEBUG=1                                     \
  -DDEBUG=1 -DTRACING=1 -DOS_POSIX=1               \
  -DIMPL_LIBXUL                                    \
  -o ../bindings.rs                                \
  -no-type-renaming                                \
  -include "$1/mozilla-config.h"                   \
  "$DIST_INCLUDE/mozilla/ServoBindings.h"          \
  -match "ServoBindings.h"                         \
  -match "nsStyleStructList.h"                     \
  $MAP_GECKO_TYPES                                 \
  $EXTRA_CODE
