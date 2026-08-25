#!/bin/bash
#
# Tests tidy for shell scripts.

set -o nounset

# Talking about some `concept in backticks` # shouldn't trigger
echo "hello world"
some_var=`echo "command substitution"`
another_var="$some_var"
if [ -z "${some_var}" ]; then
  echo "should have used [["
fi
[ -z "${another_var}" ]
# Using $@, $1 etc. shouldn't trigger.
echo "$@"
if [[ -n "$1" ]]; then
  echo "parameter 1 is $1"
fi
echo "item1 item2 item3" | awk '{$1=$1;print}'
