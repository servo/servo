#!/bin/bash
set -e

if [[ -z ${RUN_JOB+x} && $(./wpt test-jobs --includes $JOB; echo $?) -eq 0 ]] || [[ $RUN_JOB -eq 1 ]]; then
    export RUN_JOB=1
    git submodule update --init --recursive 1>&2
    export DISPLAY=:99.0
    sh -e /etc/init.d/xvfb start 1>&2
    # For uploading the manifest
    export WPT_MANIFEST_FILE=$HOME/meta/MANIFEST-$(git rev-parse HEAD).json
elif [[ -z ${RUN_JOB+x} ]]; then
    export RUN_JOB=0
fi
