#!/bin/sh

image=$1

apt-get update
apt-get install -y --no-install-recommends docker.io
docker build -t "$image" "./docker/$image/"
