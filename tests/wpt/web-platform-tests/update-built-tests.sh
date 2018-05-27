#!/usr/bin/env sh
set -ex

2dcontext/tools/build.sh
infrastructure/assumptions/tools/build.sh
html/tools/build.sh
offscreen-canvas/tools/build.sh
python mimesniff/mime-types/resources/generated-mime-types.py

# Infrastucture
python wpt make-tasks
