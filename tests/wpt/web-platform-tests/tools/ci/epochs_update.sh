#!/bin/bash
set -ex

SCRIPT_DIR=$(cd $(dirname "$0") && pwd -P)
WPT_ROOT=$SCRIPT_DIR/../..
cd $WPT_ROOT

if [ -z "$GITHUB_TOKEN" ]; then
    echo "GITHUB_TOKEN must be set as an environment variable"
    exit 1
fi

REMOTE=https://x-access-token:$GITHUB_TOKEN@github.com/web-platform-tests/wpt.git

git branch epochs/three_hourly $(./wpt rev-list --epoch 3h)
# This is safe because `git push` will by default fail for a non-fast-forward
# push, for example if the remote branch is ahead of the local branch.
git push $REMOTE epochs/three_hourly
