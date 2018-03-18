#!/bin/bash
set -ex

SCRIPT_DIR=$(dirname $(readlink -f "$0"))
WPT_ROOT=$(readlink -f $SCRIPT_DIR/../..)
cd $WPT_ROOT

source tools/ci/lib.sh

test_infrastructure() {
    local ARGS="";
    if [ $PRODUCT == "firefox" ]; then
        ARGS="--install-browser"
    else
        ARGS=$1
    fi
    ./wpt run --yes --manifest ~/meta/MANIFEST.json --metadata infrastructure/metadata/ --install-fonts $ARGS $PRODUCT infrastructure/
}

main() {
    PRODUCTS=( "firefox" "chrome" )
    for PRODUCT in "${PRODUCTS[@]}"; do
        if [ "$PRODUCT" != "firefox" ]; then
            # Firefox is expected to work using pref settings for DNS
            # Don't adjust the hostnames in that case to ensure this keeps working
            hosts_fixup
        fi
        if [ "$PRODUCT" == "chrome" ]; then
            install_chrome unstable
            test_infrastructure "--binary=$(which google-chrome-unstable)"
        else
            test_infrastructure
        fi
    done
}

main
