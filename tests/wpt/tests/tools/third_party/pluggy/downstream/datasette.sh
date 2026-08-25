#!/usr/bin/env bash
set -eux -o pipefail
if [[ ! -d datasette ]]; then
    git clone https://github.com/simonw/datasette
fi
pushd datasette && trap popd EXIT
git pull
python -m venv venv
venv/bin/pip install -e .[test] -e ../..
venv/bin/pytest
