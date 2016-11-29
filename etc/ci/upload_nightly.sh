#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

set -o errexit
set -o nounset
set -o pipefail
shopt -s failglob


usage() {
    printf "usage: ${0} android|linux|mac|macbrew|windows\n"
}


upload() {
    local -r platform="${1}"
    local -r package="${2}"
    local -r extension="${3}"

    local -r nightly_timestamp="$(date -u +"%Y-%m-%dT%H-%M-%SZ")"
    local -r nightly_filename="${nightly_timestamp}-$(basename "${package}")"
    local -r nightly_upload_dir="s3://servo-builds/nightly/${platform}"
    local -r package_upload_path="${nightly_upload_dir}/${nightly_filename}"
    s3cmd --mime-type="application/octet-stream" \
      put "${package}" "${package_upload_path}"
    s3cmd cp "${package_upload_path}" \
      "${nightly_upload_dir}/servo-latest.${extension}"

    if [[ "${platform}" == "macbrew" ]]
    then
      update_brew ${nightly_filename} ${package} ${nightly_timestamp}
    fi
}

update_brew() {
  echo "Updating brew formula"

  local -r nightly_filename="${1}"
  local -r package="${2}"
  local -r nightly_timestamp="${3}"

  local package_url sha version script_dir tmp_dir

  package_url="https://download.servo.org/nightly/macbrew/${nightly_filename}"
  sha=$(shasum -a 256 ${package} | sed -e 's/ .*//')
  version=$(echo "${nightly_timestamp}" | \
    sed -n 's/\([0-9]\{4\}\)-\([0-9]\{2\}\)-\([0-9]\{2\}\).*/\1.\2.\3/p')

  script_dir=${PWD}/$(dirname ${0})
  tmp_dir=$(mktemp -d -t homebrew-servo)
  cd ${tmp_dir}

  git clone https://github.com/servo/homebrew-servo.git
  cd homebrew-servo

  # Not using "/" as it's used in PACKAGEURL
  cat ${script_dir}/servo-binary-formula.rb.in | sed \
    "s|PACKAGEURL|${package_url}|g
     s|SHA|${sha}|g
     s|VERSION|${version}|g" > Formula/servo-bin.rb

  git add ./Formula/servo-bin.rb
  git commit -m "Version bump: ${version}"

  git push -qf \
      "https://${TOKEN}@github.com/servo/homebrew-servo.git" master \
      >/dev/null 2>&1
  rm -rf ${tmp_dir}
}

main() {
    if [[ "${#}" != 1 ]]; then
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
    elif [[ "${platform}" == "macbrew" ]]; then
        extension=tar.gz
        package=target/brew/*."${extension}"
    elif [[ "${platform}" == "windows" ]]; then
        extension=msi
        package=target/release/msi/*.msi
    else
        usage >&2
        return 1
    fi

    # Lack of quotes on package allows glob expansion
    # Note that this is not robust in the case of embedded spaces
    # TODO(aneeshusa): make this glob robust using e.g. arrays or Python
    upload "${platform}" ${package} "${extension}"
}

main "${@}"
