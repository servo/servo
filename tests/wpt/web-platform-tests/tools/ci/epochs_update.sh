#!/bin/bash
set -eux -o pipefail

SCRIPT_DIR=$(cd $(dirname "$0") && pwd -P)
WPT_ROOT=$SCRIPT_DIR/../..

EPOCHS=(
epochs/three_hourly::3h
epochs/six_hourly::6h
epochs/twelve_hourly::12h
epochs/daily::1d
epochs/weekly::1w
)

function get_epoch_branch_name () {
    echo ${1} | awk -F '::' '{print $1}'
}

function get_epoch_timeval () {
    echo ${1} | awk -F '::' '{print $2}'
}

main () {
    ALL_BRANCHES_NAMES=""
    for e in "${EPOCHS[@]}";
    do
        EPOCH=$(get_epoch_timeval ${e})
        EPOCH_BRANCH_NAME=$(get_epoch_branch_name ${e})
        EPOCH_SHA=$(./wpt rev-list --epoch ${EPOCH})
        if [ "${EPOCH_SHA}" = "" ]; then
            echo "ERROR: Empty SHA returned from ./wpt rev-list"
            exit 1
        fi
        git branch "${EPOCH_BRANCH_NAME}" "${EPOCH_SHA}"

        # Only set epoch tag if is not already tagged from a previous run.
        if ! git tag --points-at "${EPOCH_SHA}" | grep "${EPOCH_BRANCH_NAME}"; then
            EPOCH_STAMP="$(date +%Y-%m-%d_%HH)"
            git tag "${EPOCH_BRANCH_NAME}/${EPOCH_STAMP}" "${EPOCH_SHA}"
        fi

        ALL_BRANCHES_NAMES="${ALL_BRANCHES_NAMES} ${EPOCH_BRANCH_NAME}"
    done
    # This is safe because `git push` will by default fail for a non-fast-forward
    # push, for example if the remote branch is ahead of the local branch.
    git push --tags ${REMOTE} ${ALL_BRANCHES_NAMES}
}

cd $WPT_ROOT

if [ -z "$GITHUB_TOKEN" ]; then
    echo "GITHUB_TOKEN must be set as an environment variable"
    exit 1
fi

REMOTE=https://x-access-token:$GITHUB_TOKEN@github.com/web-platform-tests/wpt.git

main
