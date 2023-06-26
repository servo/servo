#!/bin/bash
set -ex

SCRIPT_DIR=$(cd $(dirname "$0") && pwd -P)
WPT_ROOT=$SCRIPT_DIR/../..
cd $WPT_ROOT

run_applicable_tox () {
    # instead of just running TOXENV (e.g., py38)
    # run all environments that start with TOXENV
    # (e.g., py38-firefox as well as py38)
    local OLD_TOXENV="$TOXENV"
    unset TOXENV
    local RUN_ENVS=$(tox -l | grep "^${OLD_TOXENV}\(\-\|\$\)" | tr "\n" ",")
    if [[ -n "$RUN_ENVS" ]]; then
        tox -e "$RUN_ENVS"
    fi
    export TOXENV="$OLD_TOXENV"
}

if ./wpt test-jobs --includes tools_unittest; then
    pip install --user -U tox
    cd tools
    run_applicable_tox
    cd $WPT_ROOT
else
    echo "Skipping tools unittest"
fi

if ./wpt test-jobs --includes wptrunner_unittest; then
    cd tools/wptrunner
    run_applicable_tox
    cd $WPT_ROOT
else
    echo "Skipping wptrunner unittest"
fi
