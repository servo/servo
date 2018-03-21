#!/bin/bash
set -ex

SCRIPT_DIR=$(dirname $(readlink -f "$0"))
WPT_ROOT=$(readlink -f $SCRIPT_DIR/../..)
cd $WPT_ROOT

main() {
    cd $WPT_ROOT
    pip install -U tox
    ./wpt install firefox browser --destination $HOME
    ./wpt install firefox webdriver --destination $HOME/firefox
    export PATH=$HOME/firefox:$PATH

    cd $WPT_ROOT/resources/test
    tox -- --binary=$HOME/browsers/firefox/firefox
}

main
