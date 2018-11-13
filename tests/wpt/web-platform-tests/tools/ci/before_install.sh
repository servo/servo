#!/bin/bash
set -e

RELEVANT_JOBS=$(./wpt test-jobs)
RELEVANT_CHANGES=$(echo "$RELEVANT_JOBS" | grep $JOB || true)
if [[ -z ${RUN_JOB+x} && ! -z $RELEVANT_CHANGES ]] || [[ $RUN_JOB -eq 1 ]]; then
    export RUN_JOB=1
    git submodule update --init --recursive 1>&2
    export DISPLAY=:99.0
    sh -e /etc/init.d/xvfb start 1>&2
    # For uploading the manifest
    export WPT_MANIFEST_FILE=$HOME/meta/MANIFEST-$(git rev-parse HEAD).json
elif [[ -z ${RUN_JOB+x} ]]; then
    export RUN_JOB=0
fi
