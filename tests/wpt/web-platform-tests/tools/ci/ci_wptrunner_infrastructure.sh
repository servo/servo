#!/bin/bash
set -ex

SCRIPT_DIR=$(cd $(dirname "$0") && pwd -P)
WPT_ROOT=$SCRIPT_DIR/../..
cd $WPT_ROOT

test_infrastructure() {
    local ARGS="";
    if [ $PRODUCT == "firefox" ]; then
        ARGS="--binary=~/build/firefox/firefox"
    else
        ARGS=$1
    fi
    TERM=dumb ./wpt run --log-mach - --yes --manifest ~/meta/MANIFEST.json --metadata infrastructure/metadata/ --install-fonts $ARGS $PRODUCT infrastructure/
}

main() {
    PRODUCTS=( "firefox" "chrome" )
    ./wpt manifest --rebuild -p ~/meta/MANIFEST.json
    for PRODUCT in "${PRODUCTS[@]}"; do
        if [[ "$PRODUCT" == "chrome" ]]; then
            test_infrastructure "--binary=$(which google-chrome-unstable) --channel dev"
        else
            test_infrastructure
        fi
    done
}

main
