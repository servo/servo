#!/bin/bash
set -ex

mkdir -p ~/meta

WPT_MANIFEST_FILE=~/meta/MANIFEST.json

./wpt manifest -p $WPT_MANIFEST_FILE
gzip -k -f --best $WPT_MANIFEST_FILE
bzip2 -k -f --best $WPT_MANIFEST_FILE
zstd -k -f --ultra -22 $WPT_MANIFEST_FILE
