#!/bin/sh

set -e
set -x

image="$1"
cd $(dirname $0)
docker build . -f "$image/Dockerfile" -t "$image"
