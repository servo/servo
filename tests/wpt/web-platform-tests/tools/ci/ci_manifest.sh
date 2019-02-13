#!/bin/bash
set -ex

SCRIPT_DIR=$(cd $(dirname "$0") && pwd -P)
WPT_ROOT=$SCRIPT_DIR/../..
cd $WPT_ROOT

mkdir -p ~/meta

python tools/ci/tag_master.py
./wpt manifest -p ~/meta/MANIFEST.json
cp ~/meta/MANIFEST.json $WPT_MANIFEST_FILE
# Force overwrite of any existing file
gzip -f $WPT_MANIFEST_FILE
