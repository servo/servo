#!/usr/bin/env sh
set -ex

# This list should be kept in sync with tools/ci/jobs.py.
conformance-checkers/tools/build.sh
html/canvas/tools/build.sh
infrastructure/assumptions/tools/build.sh
html/tools/build.sh
python3 mimesniff/mime-types/resources/generated-mime-types.py
python3 css/css-ui/tools/appearance-build-webkit-reftests.py
python3 webidl/tools/generate-setlike.py
