#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

set -o errexit
set -o nounset
set -o pipefail

REMOTE_NAME=sync-fork
LOG_FILE=test-wpt.log
CURRENT_DATE=$(date +"%d-%m-%Y")
BRANCH_NAME="wpt_update_${CURRENT_DATE}"

export GIT_AUTHOR_NAME="WPT Sync Bot"
export GIT_AUTHOR_EMAIL="josh+wptsync@joshmatthews.net"

# Retrieve the HEAD commit and extract its hash
function latest_git_commit {
    git log -1 --oneline | cut -f 1 -d ' '
}

# Create a new branch for this sync, pull down all changes from the upstream
# web-platform-tests repository, and commit the changes.
function unsafe_pull_from_upstream {
    git checkout -b "${BRANCH_NAME}" || return 1

    # Fetch all changes from upstream WPT and automatically transpose them
    # into a single servo commit.
    ./mach update-wpt --sync --no-upstream --patch || return 2
}

# Remove all local traces of this sync operation.
function cleanup {
    git remote rm "${REMOTE_NAME}" || true
    git checkout master || true
    git branch -D "${BRANCH_NAME}" || true
}

# Build Servo and run the full WPT testsuite, saving the results to a log file.
function unsafe_run_tests {
    # Perform a build that matches linux-rel-wpt
    ./mach clean-nightlies --keep 3 --force || return 1
    ./mach build --release --with-debug-assertions || return 2

    # Run the full testsuite and record the new test results.
    ./mach test-wpt --release --processes 24 --log-raw "${LOG_FILE}" \
           --always-succeed || return 3
}

# Using an existing log file, update the expected test results and amend the
# last commit with the new results.
function unsafe_update_metadata {
    ./mach update-wpt "${LOG_FILE}" || return 1
    # Ensure any new directories or ini files are included in these changes.
    git add tests/wpt/metadata || return 2
    # Merge all changes with the existing commit.
    git commit -a --amend --no-edit || return 3
}

# Push the branch to a remote branch, then open a PR for the branch
# against servo/servo.
function unsafe_open_pull_request {
    WPT_SYNC_USER=servo-wpt-sync

    # Push the changes to a remote branch owned by the bot.
    AUTH="${WPT_SYNC_USER}:${WPT_SYNC_TOKEN}"
    UPSTREAM="https://${AUTH}@github.com/${WPT_SYNC_USER}/servo.git"
    git remote add "${REMOTE_NAME}" "${UPSTREAM}" || return 1
    git push "${REMOTE_NAME}" "${BRANCH_NAME}" || return 2

    # Prepare the pull request metadata.
    cat <<EOF >prdata.json || return 3
{
  "title": "Sync WPT with upstream (${CURRENT_DATE})",
  "head": "${WPT_SYNC_USER}:${BRANCH_NAME}",
  "base": "master",
  "body":
"Automated downstream sync of changes from upstream as of ${CURRENT_DATE}.",
  "maintainer_can_modify": true
}
EOF

    # Open a pull request using the new branch.
    curl -H "Authorization: token ${WPT_SYNC_TOKEN}" \
         -H "Content-Type: application/json" \
         --data @prdata.json \
         https://api.github.com/repos/servo/servo/pulls || return 4
}

function pull_from_upstream {
    unsafe_pull_from_upstream || { code=$?; cleanup; return $code; }
}

function run_tests {
    unsafe_run_tests || { code=$?; cleanup; return $code; }
}

function update_metadata {
    unsafe_update_metadata || { code=$?; cleanup; return $code; }
}

function open_pull_request {
    unsafe_open_pull_request || { code=$?; cleanup; return $code; }
}

function run_full_sync {
    OLD_COMMIT=$(latest_git_commit)
    pull_from_upstream || { code=$?; cleanup; return $code; }
    if [[ "$(latest_git_commit)" == "${OLD_COMMIT}" ]]; then
        # No new commit was created, so there are no changes that need syncing.
        echo "No changes to sync."
        return 0
    fi
    run_tests || { code=$?; cleanup; return $code; }
    update_metadata || { code=$?; cleanup; return $code; }
    open_pull_request || { code=$?; cleanup; return $code; }
    cleanup
}
