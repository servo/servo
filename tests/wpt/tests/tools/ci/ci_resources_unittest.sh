#!/bin/bash
set -ex

SCRIPT_DIR=$(cd $(dirname "$0") && pwd -P)
WPT_ROOT=$SCRIPT_DIR/../..
cd $WPT_ROOT

main() {
    cd $WPT_ROOT
    pip install --user -U tox
    ./wpt install firefox browser --destination $HOME
    ./wpt install firefox webdriver --destination $HOME/firefox
    export PATH=$HOME/firefox:$PATH

    cd $WPT_ROOT/resources/test
    tox -- --binary=$HOME/browsers/nightly/firefox/firefox
}

main
