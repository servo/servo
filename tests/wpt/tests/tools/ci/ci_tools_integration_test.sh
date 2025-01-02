#!/bin/bash
set -e

SCRIPT_DIR=$(cd $(dirname "$0") && pwd -P)
WPT_ROOT=$SCRIPT_DIR/../..
cd $WPT_ROOT

main() {
    git fetch --quiet --unshallow https://github.com/web-platform-tests/wpt.git +refs/heads/*:refs/remotes/origin/*
    pip install --user -U tox

    # wpt commands integration tests
    cd tools/wpt
    tox
    cd $WPT_ROOT

    # WMAS test runner integration tests
    cd tools/wave
    tox
    cd $WPT_ROOT
}

main
