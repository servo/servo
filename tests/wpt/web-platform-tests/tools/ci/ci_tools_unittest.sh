#!/bin/bash
set -ex

SCRIPT_DIR=$(dirname $(readlink -f "$0"))
WPT_ROOT=$(readlink -f $SCRIPT_DIR/../..)
cd $WPT_ROOT

run_applicable_tox () {
    # instead of just running TOXENV (e.g., py27)
    # run all environments that start with TOXENV
    # (e.g., py27-firefox as well as py27)
    local OLD_TOXENV="$TOXENV"
    unset TOXENV
    local RUN_ENVS=$(tox -l | grep "^${OLD_TOXENV}\(\-\|\$\)" | tr "\n" ",")
    if [[ -n "$RUN_ENVS" ]]; then
        tox -e "$RUN_ENVS"
    fi
    export TOXENV="$OLD_TOXENV"
}


if [[ $(./wpt test-jobs --includes tools_unittest; echo $?) -eq 0 ]]; then
    pip install -U tox codecov
    cd tools
    run_applicable_tox
    cd $WPT_ROOT
else
    echo "Skipping tools unittest"
fi

if [[ $(./wpt test-jobs --includes wptrunner_unittest; echo $?) -eq 0 ]]; then
    cd tools/wptrunner
    run_applicable_tox
    cd $WPT_ROOT
else
    echo "Skipping wptrunner unittest"
fi

