#!/bin/sh

set -e
set -x

git clone https://github.com/servo/servo/
cd servo
git checkout 0a2c61da91e77102ae774075ec4126937a79f038
./mach build -d
