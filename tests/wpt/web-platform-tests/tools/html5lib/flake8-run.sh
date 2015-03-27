#!/bin/bash -e

if [[ ! -x $(which flake8) ]]; then
  echo "fatal: flake8 not found on $PATH. Exiting."
  exit 1
fi

if [[ $TRAVIS != "true" || $FLAKE == "true" ]]; then
  find html5lib/ -name '*.py' -and -not -name 'constants.py' -print0 | xargs -0 flake8 --ignore=E501
  flake1=$?
  flake8 --max-line-length=99 --ignore=E126 html5lib/constants.py
  flake2=$?
  exit $[$flake1 || $flake2]
fi
