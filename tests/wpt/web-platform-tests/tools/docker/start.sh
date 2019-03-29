#!/bin/bash

# This script is embedded in the docker image, and so the image must be updated when changes
# to the script are made. To do this, assuming you have docker installed:
# In tools/docker/ :
#   docker build .
#   docker ps # and look for the id of the image you just built
#   docker tag <image> <tag>
#   docker push <tag>
# Update the `image` specified in the project's .taskcluster.yml file


set -ex

REMOTE=${1:-https://github.com/web-platform-tests/wpt}
REF=${2:-master}

cd ~

mkdir web-platform-tests
cd web-platform-tests

git init
git remote add origin ${REMOTE}

# Initially we just fetch 50 commits in order to save several minutes of fetching
retry git fetch --quiet --depth=50 --tags origin ${REF}:task_head

git checkout --quiet task_head
