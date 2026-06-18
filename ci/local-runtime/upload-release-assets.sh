#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

set -o errexit
set -o nounset
set -o pipefail

usage() {
    cat >&2 <<'USAGE'
Usage: upload-release-assets.sh <tag> <title> <notes> <asset> [<asset> ...]

Creates the release tag when needed, then uploads the provided assets with
--clobber. Intended for local-runtime CI release artifact publication.
USAGE
}

if [[ $# -lt 4 ]]; then
    usage
    exit 64
fi

release_tag="${1}"
release_title="${2}"
release_notes="${3}"
shift 3
assets=("${@}")

: "${GITHUB_WORKSPACE:?GITHUB_WORKSPACE must be set}"
: "${GITHUB_REPOSITORY:?GITHUB_REPOSITORY must be set}"

cd "${GITHUB_WORKSPACE}"
git config --global --add safe.directory "${GITHUB_WORKSPACE}"

if ! gh release view "${release_tag}" --repo "${GITHUB_REPOSITORY}" >/dev/null 2>&1; then
    gh release create "${release_tag}" \
        --repo "${GITHUB_REPOSITORY}" \
        --title "${release_title}" \
        --notes "${release_notes}"
fi

gh release upload "${release_tag}" \
    "${assets[@]}" \
    --repo "${GITHUB_REPOSITORY}" \
    --clobber
