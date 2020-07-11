#!/usr/bin/env sh
set -ex

cd "${0%/*}"
virtualenv -p python2 .virtualenv
.virtualenv/bin/pip install genshi
git clone https://github.com/html5lib/html5lib-python.git .virtualenv/html5lib && cd .virtualenv/html5lib || cd .virtualenv/html5lib && git pull
# Pinned commit, to avoid html5lib from changing underneath us.
git reset --hard d49afd350c06339a8c59299664b8a73a3b2c3f64
git submodule update --init --recursive
cd ../..
.virtualenv/bin/pip install -e .virtualenv/html5lib
.virtualenv/bin/python update_html5lib_tests.py
