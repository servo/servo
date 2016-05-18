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


# Check for the include directory.
export DIST_INCLUDE="$1/dist/include"
if [ ! -d "$DIST_INCLUDE" ]; then
  echo "$DIST_INCLUDE: directory not found"
  exit 1
fi

export RUST_BACKTRACE=1

./rust-bindgen/target/debug/bindgen                                 \
  -o ../structs.rs                                                  \
  -x c++ -std=gnu++0x                                               \
  -allow-unknown-types                                              \
  "-I$DIST_INCLUDE" "-I$DIST_INCLUDE/nspr"                          \
  "-I$1/../nsprpub/pr/include"                                      \
  $PLATFORM_DEPENDENT_DEFINES                                       \
  -ignore-functions                                                 \
  -no-bitfield-methods                                              \
  -no-type-renaming                                                 \
  -DMOZILLA_INTERNAL_API                                            \
  -DMOZ_STYLO_BINDINGS=1                                            \
  -DJS_DEBUG=1                                                      \
  -DDEBUG=1 -DTRACING=1 -DOS_POSIX=1                                \
  -DIMPL_LIBXUL                                                     \
  -match "RefCountType.h"                                           \
  -match "nscore.h"                                                 \
  -match "nsError.h"                                                \
  -match "nsID.h"                                                   \
  -match "nsString"                                                 \
  -match "nsAString"                                                \
  -match "nsSubstring"                                              \
  -match "nsTSubstring"                                             \
  -match "nsTString"                                                \
  -match "nsISupportsBase.h"                                        \
  -match "nsCOMPtr.h"                                               \
  -match "nsIAtom.h"                                                \
  -match "nsIURI.h"                                                 \
  -match "nsAutoPtr.h"                                              \
  -match "nsColor.h"                                                \
  -match "nsCoord.h"                                                \
  -match "nsPoint.h"                                                \
  -match "nsRect.h"                                                 \
  -match "nsMargin.h"                                               \
  -match "nsCSSProperty.h"                                          \
  -match "CSSVariableValues.h"                                      \
  -match "nsFont.h"                                                 \
  -match "nsTHashtable.h"                                           \
  -match "PLDHashTable.h"                                           \
  -match "nsColor.h"                                                \
  -match "nsStyleStruct.h"                                          \
  -match "nsStyleCoord.h"                                           \
  -match "RefPtr.h"                                                 \
  -match "nsISupportsImpl.h"                                        \
  -match "gfxFontFamilyList.h"                                      \
  -match "gfxFontFeatures.h"                                        \
  -match "imgRequestProxy.h"                                        \
  -match "nsIRequest.h"                                             \
  -match "imgIRequest.h"                                            \
  -match "CounterStyleManager.h"                                    \
  -match "nsStyleConsts.h"                                          \
  -match "nsCSSValue.h"                                             \
  -match "SheetType.h"                                              \
  -match "nsIPrincipal.h"                                           \
  -match "nsDataHashtable.h"                                        \
  -match "nsCSSScanner.h"                                           \
  -match "Types.h"                                                  \
  -match "utility"                                                  \
  -match "pair"                                                     \
  -match "SheetParsingMode.h"                                       \
  -match "StaticPtr.h"                                              \
  -blacklist-type "IsDestructibleFallbackImpl"                      \
  -blacklist-type "IsDestructibleFallback"                          \
  -opaque-type "nsIntMargin"                                        \
  -opaque-type "nsIntPoint"                                         \
  -opaque-type "nsIntRect"                                          \
  -opaque-type "nsCOMArray"                                         \
  -opaque-type "nsDependentString"                                  \
  -opaque-type "EntryStore"                                         \
  -opaque-type "gfxFontFeatureValueSet"                             \
  -opaque-type "imgRequestProxy"                                    \
  -opaque-type "imgRequestProxyStatic"                              \
  -opaque-type "CounterStyleManager"                                \
  -opaque-type "ImageValue"                                         \
  -opaque-type "URLValue"                                           \
  -opaque-type "nsIPrincipal"                                       \
  -opaque-type "nsDataHashtable"                                    \
  -opaque-type "imgIRequest"                                        \
  -include "$1/mozilla-config.h"                                    \
  "$DIST_INCLUDE/nsStyleStruct.h"

if [ $? -ne 0 ]; then
  echo -e "\e[91mwarning:\e[0m bindgen exited with nonzero exit status"
else
  echo -e "\e[34minfo:\e[0m bindgen exited successfully, running tests"
  TESTS_FILE=$(mktemp)
  rustc ../structs.rs --test -o $TESTS_FILE
  $TESTS_FILE
  rm $TESTS_FILE
fi
