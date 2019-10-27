#!/bin/bash
set -ex

SCRIPT_DIR=$(cd $(dirname "$0") && pwd -P)
WPT_ROOT=$SCRIPT_DIR/../..
cd $WPT_ROOT

add_wpt_hosts() {
    ./wpt make-hosts-file | sudo tee -a /etc/hosts
}

test_infrastructure() {
    local ARGS="";
    if [ $PRODUCT == "firefox" ]; then
        ARGS="--install-browser"
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
            add_wpt_hosts
            test_infrastructure "--binary=$(which google-chrome-unstable) --channel dev"
        else
            test_infrastructure
        fi
    done
}

main
