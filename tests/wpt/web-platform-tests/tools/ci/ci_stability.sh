#!/bin/bash
set -ex

SCRIPT_DIR=$(dirname $(readlink -f "$0"))
WPT_ROOT=$(readlink -f $SCRIPT_DIR/../..)
cd $WPT_ROOT

source tools/ci/lib.sh

test_stability() {
    ./wpt check-stability $PRODUCT --output-bytes $((1024 * 1024 * 3)) --metadata ~/meta/ --install-fonts
}

main() {
    hosts_fixup
    if [ $(echo $PRODUCT | grep '^chrome:') ]; then
       install_chrome $(echo $PRODUCT | grep --only-matching '\w\+$')
    fi
    test_stability
}

main
