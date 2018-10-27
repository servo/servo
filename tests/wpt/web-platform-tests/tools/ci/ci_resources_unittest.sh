#!/bin/bash
set -ex

SCRIPT_DIR=$(cd $(dirname "$0") && pwd -P)
WPT_ROOT=$SCRIPT_DIR/../..
cd $WPT_ROOT

source tools/ci/lib.sh

main() {
    hosts_fixup

    cd $WPT_ROOT
    pip install -U tox
    ./wpt install firefox browser --destination $HOME
    ./wpt install firefox webdriver --destination $HOME/firefox
    export PATH=$HOME/firefox:$PATH

    cd $WPT_ROOT/resources/test
    tox -- --binary=$HOME/browsers/nightly/firefox/firefox
}

main
