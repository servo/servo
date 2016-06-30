#!/usr/bin/env bash

set -o errexit
set -o nounset
set -o pipefail
shopt -s failglob


usage() {
    printf "usage: ${0} android|linux|mac\n"
}


upload() {
    s3cmd put "${2}" "s3://servo-developer-preview/nightly/${1}"
}


main() {
    if [[ "$#" != 1 ]]; then
        usage >&2
        return 1
    fi

    local platform package
    platform="${1}"

    if [[ "${platform}" == "android" ]]; then
        package=target/arm-linux-androideabi/release/*.apk
    elif [[ "${platform}" == "linux" ]]; then
        package=target/release/*.tar.gz
    elif [[ "${platform}" == "mac" ]]; then
        package=target/*.dmg
    else
        usage >&2
        return 1
    fi

    # Lack of quotes on package allows glob expansion
    upload "${platform}" ${package}
}

main "$@"
