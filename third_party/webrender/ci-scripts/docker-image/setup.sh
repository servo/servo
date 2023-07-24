#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/. */

set -o errexit
set -o nounset
set -o pipefail
set -o xtrace

test "$(whoami)" == 'root'

# Install stuff we need
apt-get -y update
apt-get install -y \
    bzip2 \
    cmake \
    curl \
    gcc \
    git \
    g++ \
    libfontconfig1-dev \
    libgl1-mesa-dev \
    libx11-dev \
    openjdk-8-jdk \
    pkg-config \
    python \
    python-mako \
    python-pip \
    python-setuptools \
    python-voluptuous \
    python-yaml \
    software-properties-common

# Other stuff we need
pip install servo-tidy==0.3.0
