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

# Need to find a way to avoid hardcoding these
export RUST_BACKTRACE=1
export LIBCLANG_PATH="$(pwd)/llvm/build/Release+Asserts/lib"
export DYLD_LIBRARY_PATH="$(pwd)/llvm/build/Release+Asserts/lib"
export LD_LIBRARY_PATH="$(pwd)/llvm/build/Release+Asserts/lib"
export DIST_INCLUDE="$1/dist/include"
CLANG_SEARCH_DIRS=$(clang++ -E -x c++ - -v < /dev/null 2>&1 | awk '{ \
  if ($0 == "#include <...> search starts here:")                    \
    in_headers = 1;                                                  \
  else if ($0 == "End of search list.")                              \
    in_headers = 0;                                                  \
  else if (in_headers == 1) {                                        \
    gsub(/^[ \t]+/, "", $0);                                         \
    gsub(/[ \t].+$/, "", $0);                                        \
    printf " -isystem \"%s\"", $0;                                   \
  }
}' | sed -e s/:$//g)

# Check for the include directory.
if [ ! -d "$DIST_INCLUDE" ]; then
  echo "$DIST_INCLUDE: directory not found"
  exit 1
fi

PLATFORM_DEPENDENT_DEFINES="";
if [ "$(uname)" == "Linux" ]; then
  PLATFORM_DEPENDENT_DEFINES+="-DOS_LINUX";
else
  PLATFORM_DEPENDENT_DEFINES+="-DOS_MACOSX";
fi

# Uncomment the following line to run rust-bindgen in a debugger on mac. The
# absolute path is required to allow launching lldb with an untrusted library
# in DYLD_LIBRARY_PATH.
#
# /Applications/Xcode.app/Contents/Developer/usr/bin/lldb --
# gdb -ex "break rust_panic" -ex run  --args                          \
./rust-bindgen/target/debug/bindgen                                 \
  -x c++ -std=gnu++0x                                               \
  -allow-unknown-types                                              \
  $CLANG_SEARCH_DIRS                                                \
  "-I$DIST_INCLUDE" "-I$DIST_INCLUDE/nspr"                          \
  $PLATFORM_DEPENDENT_DEFINES                                       \
  -ignore-functions                                                 \
  -enable-cxx-namespaces                                            \
  -no-type-renaming                                                 \
  -DMOZILLA_INTERNAL_API                                            \
  -DMOZ_STYLO_BINDINGS=1                                            \
  -DDEBUG=1 -DTRACING=1 -DOS_POSIX=1                                \
  -DIMPL_LIBXUL                                                     \
  -match "nsString"                                                 \
  -match "nsAString"                                                \
  -match "nsSubstring"                                              \
  -match "nsTSubstring"                                             \
  -match "nsTString"                                                \
  -blacklist-type "nsStringComparator"                              \
  -blacklist-type "nsDefaultStringComparator"                       \
  -include "$1/mozilla-config.h"                                    \
  -o ../gecko_style_structs.rs                                      \
  "$DIST_INCLUDE/nsString.h"
yes \
  -match "nsStyleStruct"                                            \
  -match "stdint"                                                   \
  -match "nsColor"                                                  \
  -match "nsCOMPtr"                                                 \
  -match "RefPtr"                                                   \
  -match "nsIURI"                                                   \
  -match "nsCoord"                                                  \
  -match "nsStyleCoord"                                             \
  -match "nsTArray"                                                 \
  -match "nsString"                                                 \
  -match "imgIRequest"                                              \
  # "$DIST_INCLUDE/nsStyleStruct.h"
