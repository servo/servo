#!/bin/bash

# Retries a given command until it passes
# Run as `retry.sh N command args...`
# where `N` is the maximum  number of tries, and `command args...` is the
# command to run, with arguments

n=$1
shift; # this removes the first argument from $@
for i in `seq $n`; do
        echo "====== RUN NUMBER: $i ======";
        if $@ # run command, check if exit code is zero
        then
                exit 0 # command passed, all is well
        fi
done
exit 1
