#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

set -o errexit
set -o nounset
set -o pipefail

# Only check for issues with a `/` in the title. This is to avoid accidentally closing
# intermittency issues that are not related to WPT (where we know that it always contains
# a `/` at the start of the WPT test location).
#
# Additionally, any issue that was created in the last week is ignored. To give it some time
# before intermittency shows up.
INTERMITTENT_ISSUES=$(gh api /repos/servo/servo/issues?labels="I-intermittent" --paginate | \
    jq '.[] | select(.title | test("/")) | select(.updated_at | fromdate | . < now - (60 * 60 * 24 * 7)) | .number')

NOW=$(date -u "+%s")
NOW_LAST_MONTH=$(( ${NOW} - (60 * 60 * 24 * 31)))
for issue_id in ${INTERMITTENT_ISSUES};
do
  last_update=$(gh api /repos/servo/servo/issues/${issue_id}/timeline --paginate --jq 'map(select(.event ==  "cross-referenced")) | sort_by(.updated_at) | last.updated_at')
  if [[ -z "${last_update}" ]]; then
    echo "https://github.com/servo/servo/issues/${issue_id} has not received any updates"
  else
    # We paginate results. This could mean that there are multiple pages of timeline events
    # Therefore, we need to take the last one of these. If there is only 1 update then
    # it implicitly is the last item
    last_update_as_array=(${last_update})
    last_update=${last_update_as_array[-1]}

    last_update_timestamp=$(date -u --date="${last_update}" "+%s")
    if (( "${last_update_timestamp}" < "${NOW_LAST_MONTH}" )); then
      echo "https://github.com/servo/servo/issues/${issue_id} has not received updates for over a month"
    fi
  fi
done
