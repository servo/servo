#!/bin/sh
curl \
    --retry 5 \
    --connect-timeout 10 \
    --location
    https://queue.taskcluster.net/v1/task/$1/artifacts/$2
