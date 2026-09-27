#!/bin/bash
set -e

SCRIPT_DIR=$(cd $(dirname "$0") && pwd -P)
WPT_ROOT=$SCRIPT_DIR/../..
cd $WPT_ROOT

main() {
    git fetch --quiet --unshallow https://github.com/web-platform-tests/wpt.git +refs/heads/*:refs/remotes/origin/*

    # wpt commands integration tests
    cd $WPT_ROOT/tools/wpt
    pip install -U tox
    tox

    # WMAS test runner integration tests
    cd $WPT_ROOT/tools/wave
    pip install -U tox
    tox
}

main
