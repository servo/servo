#!/usr/bin/env sh
set -ex

cd "${0%/*}"
virtualenv -p python .virtualenv
.virtualenv/bin/pip install pyyaml cairocffi
.virtualenv/bin/python gentest.py
