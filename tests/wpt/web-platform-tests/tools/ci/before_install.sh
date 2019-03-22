#!/bin/bash
set -e

export GITHUB_PULL_REQUEST=$TRAVIS_PULL_REQUEST
export GITHUB_BRANCH=$TRAVIS_BRANCH

if [[ $RUN_JOB -eq 1 ]] || ./wpt test-jobs --includes $JOB; then
    export RUN_JOB=1
    git submodule update --init --recursive 1>&2
    export DISPLAY=:99.0
    sh -e /etc/init.d/xvfb start 1>&2
    # For uploading the manifest
    export WPT_MANIFEST_FILE=$HOME/meta/MANIFEST-$(git rev-parse HEAD).json
else
    export RUN_JOB=0
fi
