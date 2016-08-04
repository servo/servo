#!/bin/bash
#
# Tests tidy for shell scripts.

set -o nounset

# Talking about some `concept in backticks` # shouldn't trigger
echo "hello world"
some_var=`echo "command substitution"`
