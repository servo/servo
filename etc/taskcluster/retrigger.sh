
#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

set -o errexit
set -o nounset
set -o pipefail

# Retrigger a given set of taskcluster tasks a given number of times each.
# eg. `./retrigger.sh 10 THkZut0wRq-SAmDSKIQGjg LFa41aanSHyLzlodxlgA9w` will
# cause 20 new tasks to be scheduled for the associated task group(s).

export TASKCLUSTER_ROOT_URL=https://community-tc.services.mozilla.com/
eval $(taskcluster signin)
times=$1
shift
for task in $@
do
    for (( i = 1 ; i < $times; i++ ))
    do
        taskcluster task retrigger $task
    done
done
