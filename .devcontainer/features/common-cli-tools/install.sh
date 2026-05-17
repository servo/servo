#!/usr/bin/env bash

set -euo pipefail

export DEBIAN_FRONTEND=noninteractive

# The tools we install here are not strictly required for developing servo,
# but they may be useful when interactively developing.
# We keep this seperate from the main Dockerfile, since installing this should be fast,
# and we do want to have a minimal image of sorts.
apt-get update
apt-get install -y --no-install-recommends \
    bat \
    fd-find \
    file \
    less \
    ripgrep \
    xxd
rm -rf /var/lib/apt/lists/*

# Ubuntu packages expose some tools under Debian-specific names.
ln -sf /usr/bin/batcat /usr/local/bin/bat
ln -sf /usr/bin/fdfind /usr/local/bin/fd
