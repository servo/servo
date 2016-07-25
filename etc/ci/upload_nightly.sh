#!/usr/bin/env bash

set -o errexit
set -o nounset
set -o pipefail
shopt -s failglob


usage() {
    printf "usage: ${0} android|linux|mac|windows\n"
}


upload() {
    local package_filename 
    package_filename="$(basename "${2}")"
    local -r nightly_upload_dir="s3://servo-builds/nightly/${1}"
    local -r package_upload_path="${nightly_upload_dir}/${package_filename}"
    s3cmd --mime-type="application/octet-stream" \
          put "${2}" "${package_upload_path}"
    s3cmd cp "${package_upload_path}" "${nightly_upload_dir}/servo-latest.${3}"
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
        extension=msi
        package=target/msi/*.msi
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
