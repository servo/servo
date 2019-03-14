#!/bin/bash
set -ex

SCRIPT_DIR=$(cd $(dirname "$0") && pwd -P)
WPT_ROOT=$SCRIPT_DIR/../..
cd $WPT_ROOT

mkdir -p ~/meta
./wpt manifest -p ~/meta/MANIFEST.json
./wpt lint --all
