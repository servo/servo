#!/bin/bash
set -eux -o pipefail

SCRIPT_DIR=$(cd $(dirname "$0") && pwd -P)
WPT_ROOT=$SCRIPT_DIR/../..

main () {
    # Find the latest version of the package to install.
    VERSION=$(npm info @webref/idl version)

    # Install @webref/idl in a temporary directory.
    TMPDIR=$(mktemp -d)
    cd $TMPDIR
    npm install @webref/idl@$VERSION

    # Delete interfaces/*.idl except tentative ones
    cd $WPT_ROOT
    find interfaces/ -name '*.idl' -not -name '*.tentative.idl' -delete

    # Handle cssom.idl with preamble first.
    cat <<EOF > interfaces/cssom.idl
// GENERATED PREAMBLE - DO NOT EDIT
// CSSOMString is an implementation-defined type of either DOMString or
// USVString in CSSOM: https://drafts.csswg.org/cssom/#cssomstring-type
// For web-platform-tests, use DOMString because USVString has additional
// requirements in type conversion and could result in spurious failures for
// implementations that use DOMString.
typedef DOMString CSSOMString;

EOF
    cat $TMPDIR/node_modules/@webref/idl/cssom.idl >> interfaces/cssom.idl
    rm $TMPDIR/node_modules/@webref/idl/cssom.idl

    # Move remaining *.idl from @webref/idl to interfaces/
    mv $TMPDIR/node_modules/@webref/idl/*.idl interfaces/

    # Cleanup
    rm -rf $TMPDIR

    if [ -n "$GITHUB_ENV" ]; then
        echo webref_idl_version=$VERSION >> $GITHUB_ENV
    fi
}

main
