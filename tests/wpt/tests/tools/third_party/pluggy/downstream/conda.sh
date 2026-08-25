#!/usr/bin/env bash
set -eux -o pipefail
if [[ ! -d conda ]]; then
    git clone https://github.com/conda/conda
fi
pushd conda && trap popd EXIT
git pull
set +eu
source dev/start
set -eu
pip install -e ../../
pytest -m "not integration and not installed"
