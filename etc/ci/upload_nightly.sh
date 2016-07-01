#!/usr/bin/env bash

set -o errexit
set -o nounset
set -o pipefail
shopt -s failglob


usage() {
    printf "usage: ${0} android|linux|mac|windows\n"
}


upload() {
    local s3path filename
    s3path="s3://servo-builds/nightly"
    filename=test=$(basename "${2}")
    
    s3cmd put "${s3path}/${2}" "${s3path}/${1}/"
    s3cmd cp "${s3path}/${1}/${filename}" "${s3path}/${1}/servo-latest.${3}"
}


main() {
    if [[ "$#" != 1 ]]; then
        usage >&2
        return 1
    fi

    local platform package extension
    platform="${1}"

    if [[ "${platform}" == "android" ]]; then
        extension=apk
        package=target/arm-linux-androideabi/release/*."${extension}"
    elif [[ "${platform}" == "linux" ]]; then
        extension=tar.gz
        package=target/*."${extension}"
    elif [[ "${platform}" == "mac" ]]; then
        extension=dmg
        package=target/*."${extension}"
    elif [[ "${platform}" == "windows" ]]; then
        extension=tar.gz
        package=target/*."${extension}"
    else
        usage >&2
        return 1
    fi

    # Lack of quotes on package allows glob expansion
    # Note that this is not robust in the case of embedded spaces
    # TODO(aneeshusa): make this glob robust using e.g. arrays or Python
    upload "${platform}" ${package} "${extension}"
}

main "$@"
