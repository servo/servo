#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

set -o errexit
set -o nounset
set -o pipefail
shopt -s failglob


usage() {
    printf \
        "usage: %s android|linux|mac|macbrew|windows-msvc\n" \
        "${0}"
}


upload() {
    local -r platform="${1}"
    local -r nightly_local_path="${2}"
    local -r extension="${3}"
    local -r timestamp="${4}"
    local nightly_filename
    nightly_filename="${timestamp}-$(basename "${nightly_local_path}")"
    local -r nightly_upload_dir="s3://servo-builds/nightly/${platform}"
    local -r nightly_upload_path="${nightly_upload_dir}/${nightly_filename}"
    local -r checksum_upload_path="${nightly_upload_path}-checksum.txt"

    s3cmd --mime-type="application/octet-stream" \
        put "${nightly_local_path}" "${nightly_upload_path}"
    sha384sum "${nightly_local_path}" | s3cmd --mime-type="text/plain" \
        put - "${checksum_upload_path}"
    s3cmd cp \
        "${nightly_upload_path}" \
        "${nightly_upload_dir}/servo-latest.${extension}"
    s3cmd cp \
        "${checksum_upload_path}" \
        "${nightly_upload_dir}/servo-latest.${extension}-checksum.txt"
}

update_brew() {
  echo "Updating brew formula"

  local package_url sha version script_dir tmp_dir nightly_filename

  nightly_filename="${2}-$(basename "${1}")"
  package_url="https://download.servo.org/nightly/macbrew/${nightly_filename}"
  sha="$(shasum -a 256 "${1}" | sed -e 's/ .*//')"

  # This will transform a timestamp (2016-12-13T08-01-10Z for example)
  # into a valid brew version number (2016.12.13).
  version="$(echo "${2}" | \
    sed -n 's/\([0-9]\{4\}\)-\([0-9]\{2\}\)-\([0-9]\{2\}\).*/\1.\2.\3/p')"

  script_dir="${PWD}/$(dirname "${0}")"
  tmp_dir="$(mktemp -d -t homebrew-servo.XXXXX)"

  git -C "${tmp_dir}" clone https://github.com/servo/homebrew-servo.git .

  # Not using "/" as it's used in PACKAGEURL
  sed "s|PACKAGEURL|${package_url}|g
       s|SHA|${sha}|g
       s|VERSION|${version}|g" \
       < "${script_dir}/servo-binary-formula.rb.in" \
       > "${tmp_dir}/Formula/servo-bin.rb"

  git -C "${tmp_dir}" add ./Formula/servo-bin.rb
  git -C "${tmp_dir}" \
      -c user.name="Tom Servo" \
      -c user.email="servo@servo.org" \
      commit \
      --message="Version bump: ${version}"

  git -C "${tmp_dir}" push -qf \
      "https://${GITHUB_HOMEBREW_TOKEN}@github.com/servo/homebrew-servo.git" \
      master >/dev/null 2>&1

  rm -rf "${tmp_dir}"
}

main() {
    if (( "${#}" != 1 )); then
        usage >&2
        return 1
    fi

    local platform package extension nightly_timestamp
    platform="${1}"
    nightly_timestamp="$(date -u +"%Y-%m-%dT%H-%M-%SZ")"

    if [[ "${platform}" == "android" ]]; then
        extension=apk
        package=target/arm-linux-androideabi/release/*."${extension}"
    elif [[ "${platform}" == "linux" ]]; then
        extension=tar.gz
        package=target/release/*."${extension}"
    elif [[ "${platform}" == "mac" ]]; then
        extension=dmg
        package=target/release/*."${extension}"
    elif [[ "${platform}" == "macbrew" ]]; then
        extension=tar.gz
        package=target/release/brew/*."${extension}"
    elif [[ "${platform}" == "windows-msvc" ]]; then
        extension=msi
        package=target/release/msi/*.msi
        upload "${platform}" ${package} "${extension}" "${nightly_timestamp}"
        extension=zip
        package=target/release/msi/*.zip
    else
        usage >&2
        return 1
    fi

    # Lack of quotes on package allows glob expansion
    # Note that this is not robust in the case of embedded spaces
    # TODO(aneeshusa): make this glob robust using e.g. arrays or Python
    upload "${platform}" ${package} "${extension}" "${nightly_timestamp}"

    if [[ "${platform}" == "macbrew" ]]; then
      update_brew ${package} "${nightly_timestamp}"
    fi
}

main "${@}"
