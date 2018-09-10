#!/bin/sh

apt-get update -q
apt-get install -qy --no-install-recommends "$@"

# Purge apt-get caches to minimize image size
apt-get auto-remove -y
apt-get clean -y
rm -rf /var/lib/apt/lists/
