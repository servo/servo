#!/usr/bin/env sh
set -ex

cd "${0%/*}"
for script in *.py; do
    python3 "$script"
done
