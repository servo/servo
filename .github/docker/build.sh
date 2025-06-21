#!/usr/bin/env bash

cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null || exit 1

#docker build \
# --label "org.opencontainers.image.source=https://github.com/servo/servo" \
# --label "org.opencontainers.image.description=Docker image for servo CI" \
# --target servo_cooked_dev-crown-default_features \
# --tag ghcr.io/jschwe/servo_ci_testing_dev-crown-default_features:latest \
# .

docker build \
 --platform linux/amd64 \
 --label "org.opencontainers.image.source=https://github.com/servo/servo" \
 --label "org.opencontainers.image.description=Docker image for servo CI" \
 --tag ghcr.io/jschwe/servo_msrv_ci:latest \
 .