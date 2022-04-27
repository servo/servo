# Build helper binaries for musllinux tests.
# Usages:
#   build.sh  # Build everything.
#   build.sh $DISTRO $ARCH  # Build one executable in $ARCH using $DISTRO.
#
# Either invocation ultimately runs this script in a Docker container with
# `build.sh glibc|musl $ARCH` to actually build the executable.

set -euo pipefail
set -x

UBUNTU_VERSION='focal'
ALPINE_VERSION='v3.13'

build_one_in_ubuntu () {
    $1 "multiarch/ubuntu-core:${2}-${UBUNTU_VERSION}" \
        bash "/home/hello-world/musllinux/build.sh" glibc "glibc-${2}"
}

build_one_in_alpine () {
    $1 "multiarch/alpine:${2}-${ALPINE_VERSION}" \
        sh "/home/hello-world/musllinux/build.sh" musl "musl-${2}"
}

build_in_container () {
    local SOURCE="$(dirname $(dirname $(realpath ${BASH_SOURCE[0]})))"
    DOCKER="docker run --rm -v ${SOURCE}:/home/hello-world"

    if [[ $# -ne 0 ]]; then
        "build_one_in_${1}" "$DOCKER" "$2"
        return
    fi

    build_one_in_alpine "$DOCKER" x86_64
    build_one_in_alpine "$DOCKER" i386
    build_one_in_alpine "$DOCKER" aarch64
    build_one_in_ubuntu "$DOCKER" x86_64
}

if [[ $# -eq 0 ]]; then
    build_in_container
    exit 0
elif [[ "$1" == "glibc" ]]; then
    DEBIAN_FRONTEND=noninteractive apt-get update -qq \
        && apt-get install -qqy --no-install-recommends gcc libc6-dev
elif [[ "$1" == "musl" ]]; then
    apk add -q build-base
else
    build_in_container "$@"
    exit 0
fi

build () {
    local CFLAGS=""
    local OUT="/home/hello-world/musllinux/${2}"
    gcc -Os ${CFLAGS} -o "${OUT}-full" "/home/hello-world/hello-world.c"
    head -c1024 "${OUT}-full" > "$OUT"
    rm -f "${OUT}-full"
}

build "$@"
