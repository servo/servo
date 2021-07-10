#!/usr/bin/env sh
set -ex

cd "${0%/*}"
virtualenv -p python .virtualenv
.virtualenv/bin/pip install fonttools==3.13.1
.virtualenv/bin/python ahem-generate-table.py
