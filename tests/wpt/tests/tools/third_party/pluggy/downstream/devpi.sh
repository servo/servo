#!/usr/bin/env bash
set -eux -o pipefail
if [[ ! -d devpi ]]; then
    git clone https://github.com/devpi/devpi
fi
pushd devpi && trap popd EXIT
git pull
python -m venv venv
venv/bin/pip install -r dev-requirements.txt -e ../..
venv/bin/pytest common
venv/bin/pytest server
venv/bin/pytest client
venv/bin/pytest web
