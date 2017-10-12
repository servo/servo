#!/bin/bash
set -e

if [[ $(./wpt test-jobs --includes $JOB; echo $?) -eq 0 ]]; then
    export RUN_JOB=1
    git submodule update --init --recursive 1>&2
    export DISPLAY=:99.0
    sh -e /etc/init.d/xvfb start 1>&2
else
    export RUN_JOB=0
fi
