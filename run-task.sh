#!/bin/sh

set -e
set -x

./curl-artifact.sh $BUILD_TASK_ID public/executable.gz -o executable.gz
gunzip executable.gz
chmod +x executable
./executable
