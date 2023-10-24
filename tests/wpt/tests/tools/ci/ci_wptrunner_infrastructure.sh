#!/bin/bash
set -ex

SCRIPT_DIR=$(cd $(dirname "$0") && pwd -P)
WPT_ROOT=$SCRIPT_DIR/../..
cd $WPT_ROOT

run_infra_test() {
    TERM=dumb ./wpt run --log-mach - --yes --manifest ~/meta/MANIFEST.json --metadata infrastructure/metadata/ --install-fonts --install-webdriver --log-wptreport="/home/test/artifacts/wptreport-$1.json" $2 $1 infrastructure/
}

main() {
    ./wpt manifest --rebuild -p ~/meta/MANIFEST.json
    run_infra_test "chrome" "--binary=$(which google-chrome-unstable) --enable-swiftshader --channel dev $1"
    run_infra_test "firefox" "--binary=~/build/firefox/firefox $1"
    run_infra_test "firefox_android" "--install-browser --logcat-dir=/home/test/artifacts/ $1"
}

main $1
