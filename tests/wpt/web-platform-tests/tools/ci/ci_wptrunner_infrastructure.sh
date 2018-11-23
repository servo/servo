#!/bin/bash
set -ex

SCRIPT_DIR=$(cd $(dirname "$0") && pwd -P)
WPT_ROOT=$SCRIPT_DIR/../..
cd $WPT_ROOT

source tools/ci/lib.sh

test_infrastructure() {
    local ARGS="";
    if [ $PRODUCT == "firefox" ]; then
        ARGS="--install-browser"
    else
        ARGS=$1
    fi
    ./wpt run --log-tbpl - --yes --manifest ~/meta/MANIFEST.json --metadata infrastructure/metadata/ --install-fonts $ARGS $PRODUCT infrastructure/
}

main() {
    PRODUCTS=( "firefox" "chrome" )
    ./wpt manifest --rebuild -p ~/meta/MANIFEST.json
    for PRODUCT in "${PRODUCTS[@]}"; do
        if [ "$PRODUCT" != "firefox" ]; then
            # Firefox is expected to work using pref settings for DNS
            # Don't adjust the hostnames in that case to ensure this keeps working
            hosts_fixup
        fi
        if [[ "$PRODUCT" == "chrome" ]]; then
            install_chrome unstable
            test_infrastructure "--binary=$(which google-chrome-unstable)"
        else
            test_infrastructure
        fi
    done
}

main
