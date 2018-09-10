#!/bin/sh

set -e
set -x

image="$1"

apt-get update -q
apt-get install -qy --no-install-recommends docker.io
docker version
./docker/build.sh "$image"
