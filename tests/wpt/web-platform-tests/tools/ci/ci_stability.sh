#!/bin/bash
set -ex

SCRIPT_DIR=$(dirname $(readlink -f "$0"))
WPT_ROOT=$(readlink -f $SCRIPT_DIR/../..)
cd $WPT_ROOT

source tools/ci/lib.sh

test_stability() {
    local extra_arg=$1
    ./wpt check-stability $PRODUCT $extra_arg --output-bytes $((1024 * 1024 * 3)) --metadata ~/meta/ --install-fonts
}

main() {
    hosts_fixup
    local extra_arg=""
    if [ $(echo $PRODUCT | grep '^chrome:') ]; then
        local channel=$(echo $PRODUCT | grep --only-matching '\w\+$')
        if [[ $channel == "dev" ]]; then
            # The package name for Google Chrome Dev uses "unstable", not "dev".
            channel="unstable"
        fi
        install_chrome $channel
        extra_arg="--binary=$(which google-chrome-$channel)"
    fi
    test_stability $extra_arg
}

main
