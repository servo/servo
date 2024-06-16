#!/usr/bin/env bash
set -eux -o pipefail
if [[ ! -d hatch ]]; then
    git clone https://github.com/pypa/hatch
fi
pushd hatch && trap popd EXIT
git pull
python -m venv venv
venv/bin/pip install -e . -e ./backend  -e ../..
venv/bin/hatch run dev
