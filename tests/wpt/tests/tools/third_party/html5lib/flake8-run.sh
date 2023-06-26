#!/bin/bash -e

if [[ ! -x $(which flake8) ]]; then
  echo "fatal: flake8 not found on $PATH. Exiting."
  exit 1
fi

flake8 `dirname $0`
exit $?
