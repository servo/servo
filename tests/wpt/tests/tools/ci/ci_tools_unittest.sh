#!/bin/bash
set -ex

SCRIPT_DIR=$(cd $(dirname "$0") && pwd -P)
WPT_ROOT=$SCRIPT_DIR/../..
cd $WPT_ROOT

if ./wpt test-jobs --includes tools_unittest; then
    pip install --user -U tox
    cd tools
    tox -f "$TOXENV"
    cd $WPT_ROOT
else
    echo "Skipping tools unittest"
fi

if ./wpt test-jobs --includes wptrunner_unittest; then
    cd tools/wptrunner
    tox -f "$TOXENV"
    cd $WPT_ROOT
else
    echo "Skipping wptrunner unittest"
fi
