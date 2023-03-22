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
