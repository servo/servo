#!/bin/bash

set -ex

REMOTE=${1:-https://github.com/web-platform-tests/wpt}
REF=${2:-master}

cd ~

if [ -e /dev/kvm ]; then
    # If kvm is present ensure that the test user can access it
    # Ideally this could be done by adding the test user to the
    # owning group, but then we need to re-login to evaluate the
    # group membership. This chmod doesn't affect the host.
    sudo chmod a+rw /dev/kvm
fi

if [ ! -d web-platform-tests ]; then
    mkdir web-platform-tests
    cd web-platform-tests

    git init
    git remote add origin ${REMOTE}

    # Initially we just fetch 50 commits in order to save several minutes of fetching
    retry git fetch --quiet --depth=50 --tags origin ${REF}:task_head

    git checkout --quiet task_head
fi
