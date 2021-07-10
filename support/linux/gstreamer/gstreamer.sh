#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

set -o errexit

curl -L https://servo-deps-2.s3.amazonaws.com/gstreamer/gstreamer-1.16-x86_64-linux-gnu.20190515.tar.gz | tar xz
sed -i "s;prefix=/opt/gst;prefix=$PWD/gst;g" $PWD/gst/lib/pkgconfig/*.pc
