#!/bin/sh

image="$1"

apt-get update
apt-get install -y --no-install-recommends docker.io
docker version
./docker/build.sh "$image"
