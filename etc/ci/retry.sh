#!/usr/bin/env bash

# Retries a given command until it passes
# Run as `retry.sh N command args...`
# where `N` is the maximum  number of tries, and `command args...` is the
# command to run, with arguments

set -o errexit
set -o nounset
set -o pipefail

n="$1"
shift; # this removes the first argument from $@
for i in $(seq $n); do
        echo "====== RUN NUMBER: $i ======";
        # Run command and exit success if return code is 0, else ignore it
        "$@" && exit 0 || true
done
exit 1
