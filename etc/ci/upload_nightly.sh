#!/usr/bin/env bash

set -o errexit
set -o nounset
set -o pipefail
shopt -s failglob


usage() {
    printf "usage: ${0} android|linux|mac|windows\n"
}


upload() {
    s3cmd put "${2}" "s3://servo-builds/nightly/${1}/"
    s3cmd put "${2}" "s3://servo-builds/nightly/${1}/servo-latest${3}"
}


main() {
    if [[ "$#" != 1 ]]; then
        usage >&2
        return 1
    fi

    local platform package ext
    platform="${1}"

    if [[ "${platform}" == "android" ]]; then
        package=target/arm-linux-androideabi/release/*.apk
        ext=".apk"
    elif [[ "${platform}" == "linux" ]]; then
        package=target/*.tar.gz
        ext=".tar.gz"
    elif [[ "${platform}" == "mac" ]]; then
        package=target/*.dmg
        ext=".dmg"
    elif [[ "${platform}" == "windows" ]]; then
        package=target/*.tar.gz
        ext=".tar.gz"
    else
        usage >&2
        return 1
    fi

    # Lack of quotes on package allows glob expansion
    # Note that this is not robust in the case of embedded spaces
    # TODO(aneeshusa): make this glob robust using e.g. arrays or Python
    upload "${platform}" ${package} ${ext}
}

main "$@"
