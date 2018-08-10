#!/usr/bin/env bash

# This script is invoked as follows: the first argument is the target branch
# for the backport. All following arguments are considered the "commit spec",
# and will be passed to cherry-pick.

TARGET_BRANCH="$1"
PR_BRANCH="backport-${TARGET_BRANCH}"
COMMIT_SPEC="${@:2}"

if ! git checkout "$TARGET_BRANCH"; then
    echo "Failed to checkout $TARGET_BRANCH"
    exit 1
fi

if ! git pull --ff-only; then
    echo "Unable to update $TARGET_BRANCH"
    exit 2

if ! git checkout -b "$PR_BRANCH"; then
    echo "Failed to open new branch $PR_BRANCH"
    exit 3
fi

if ! git cherry-pick -x $COMMIT_SPEC; then
    echo "Cherry-pick failed. Please fix up manually."
else
    echo "Clean backport. Add changelog and open PR."
fi

