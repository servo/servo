#!/usr/bin/env bash
set -eux -o pipefail
if [[ ! -d tox ]]; then
    git clone https://github.com/tox-dev/tox
fi
pushd tox && trap popd EXIT
python -m venv venv
venv/bin/pip install -e .[testing] -e ../..
venv/bin/pytest
