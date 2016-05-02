#!/usr/bin/env bash

set -o errexit
set -o nounset
set -o pipefail

diff="$(git diff -- */*/Cargo.lock)"
echo "$diff"
[[ ! $diff ]]
