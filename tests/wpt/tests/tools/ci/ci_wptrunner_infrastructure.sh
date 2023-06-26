#!/bin/bash
set -ex

SCRIPT_DIR=$(cd $(dirname "$0") && pwd -P)
WPT_ROOT=$SCRIPT_DIR/../..
cd $WPT_ROOT

test_infrastructure() {
    TERM=dumb ./wpt run --log-mach - --yes --manifest ~/meta/MANIFEST.json --metadata infrastructure/metadata/ --install-fonts --install-webdriver $1 $PRODUCT infrastructure/
}

main() {
    PRODUCTS=( "firefox" "chrome" )
    ./wpt manifest --rebuild -p ~/meta/MANIFEST.json
    for PRODUCT in "${PRODUCTS[@]}"; do
        if [[ "$PRODUCT" == "chrome" ]]; then
            # Taskcluster machines do not have GPUs, so use software rendering via --enable-swiftshader.
            test_infrastructure "--binary=$(which google-chrome-unstable) --enable-swiftshader --channel dev" "$1"
        else
            test_infrastructure "--binary=~/build/firefox/firefox" "$1"
        fi
    done
}

main $1
