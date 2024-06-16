#!/usr/bin/env bash
set -eux -o pipefail
if [[ ! -d pytest ]]; then
    git clone https://github.com/pytest-dev/pytest
fi
pushd pytest && trap popd EXIT
git pull
python -m venv venv
venv/bin/pip install -e .[testing] -e ../..
venv/bin/pytest
