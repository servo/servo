#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

set -o errexit

wget https://github.com/ferjm/gstreamer-1.14.1-ubuntu-trusty/raw/master/gstreamer.tar.gz -O gstreamer.tar.gz
tar -zxf gstreamer.tar.gz
rm gstreamer.tar.gz
sed -i "s;prefix=/root/gstreamer;prefix=${PWD}/gstreamer;g" ${PWD}/gstreamer/lib/x86_64-linux-gnu/pkgconfig/*.pc
