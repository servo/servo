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

RELEVANT_JOBS=$(./wpt test-jobs)

RELEVANT_CHANGES_TOOLS=$(echo "$RELEVANT_JOBS" | grep "tools_unittest" || true)
if [[ ! -z $RELEVANT_CHANGES_TOOLS ]]; then
    pip install -U tox codecov
    cd tools
    run_applicable_tox
    cd $WPT_ROOT
else
    echo "Skipping tools unittest"
fi

RELEVANT_CHANGES_WPTRUNNER=$(echo "$RELEVANT_JOBS" | grep "wptrunner_unittest" || true)
if [[ ! -z $RELEVANT_CHANGES_WPTRUNNER ]]; then
    cd tools/wptrunner
    run_applicable_tox
    cd $WPT_ROOT
else
    echo "Skipping wptrunner unittest"
fi

