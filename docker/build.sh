#!/bin/sh

set -e
set -x

image="$1"
docker build -t "$image" "$(dirname $0)/$image"
