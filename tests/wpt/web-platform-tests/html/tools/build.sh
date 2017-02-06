#!/usr/bin/env sh
set -ex

cd "${0%/*}"
virtualenv -p python .virtualenv
.virtualenv/bin/pip install genshi
git clone https://github.com/html5lib/html5lib-python.git .virtualenv/html5lib && cd .virtualenv/html5lib || cd .virtualenv/html5lib && git pull
git submodule update --init --recursive
cd ../..
.virtualenv/bin/pip install -e .virtualenv/html5lib
.virtualenv/bin/python update_html5lib_tests.py
