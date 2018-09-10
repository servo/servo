#!/bin/sh

set -e
set -x

cd $(dirname $0)
python2 -m pip install -r requirements.txt --target vendored
