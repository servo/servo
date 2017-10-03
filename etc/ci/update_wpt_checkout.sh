#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

set -o errexit
set -o nounset
set -o pipefail

WPT_SYNC_USER=servo-wpt-sync

# Perform a build that matches linux-rel-wpt
./mach clean-nightlies --keep 3 --force
./mach build --release --with-debug-assertions

CURRENT_DATE=`date +"%d-%m-%Y"`
BRANCH_NAME="wpt_update_${CURRENT_DATE}"
git checkout -b $BRANCH_NAME

export GIT_AUTHOR_NAME="WPT Sync Bot"
export GIT_AUTHOR_EMAIL="josh+wptsync@joshmatthews.net"

OLD_COMMIT=`git log -1 --oneline | cut -f 1 -d ' '`

# Fetch all changes from upstream WPT and automatically transpose them into a single servo commit.
./mach update-wpt --sync --no-upstream

NEW_COMMIT=`git log -1 --oneline | cut -f 1 -d ' '`
if [ "$OLD_COMMIT" -e "$NEW_COMMIT" ];
then
    # No new commit was created, so there are no changes that need syncing.
    echo "No changes to sync."
    git checkout master
    exit 0
fi

# Run the full testsuite and record the new test results.
./mach test-wpt --release --processes 24 --log-raw test-wpt.log --always-succeed
./mach update-wpt test-wpt.log --no-patch
# Ensure that any new directories or ini files are included in these changes.
git add tests/wpt/metadata
# Merge all changes with the existing commit.
git commit -a --amend --no-edit

# Push the changes to a remote branch owned by the bot.
git remote add sync-fork "https://${WPT_SYNC_USER}:${WPT_SYNC_TOKEN}@github.com/${WPT_SYNC_USER}/servo.git"
git push sync-fork $BRANCH_NAME

# Prepare the pull request metadata.
cat <<EOF >prdata.json
{
  "title": "Sync WPT with upstream (${CURRENT_DATE})",
  "head": "${WPT_SYNC_USER}:${BRANCH_NAME}",
  "base": "master",
  "body": "Automated downstream sync of changes from upstream as of ${CURRENT_DATE}.",
  "maintainer_can_modify": true
}
EOF

# Open a pull request using the new branch.
curl --user "${WPT_SYNC_USER" \
     -H "Authorization: token ${WPT_SYNC_TOKEN}" \
     -H "Content-Type: application/json" \
     --data @prdata.json \
     https://api.github.com/repos/servo/servo/pulls

# Remove all local traces of this sync operation.
git remote rm sync-fork

git checkout master
git branch -D $BRANCH_NAME
