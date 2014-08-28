#!/bin/bash

set -e

case ${TRAVIS_OS_NAME} in
    linux)
        /usr/bin/Xorg :1 -noreset +extension GLX +extension RANDR +extension RENDER -logfile ./xorg.log -config etc/ci/xorg.conf &

        # Patch the broken font config files on ubuntu 12.04 lts - this should be removed when travis moves to ubuntu 14.04 lts
        sudo cp etc/ci/fontconfig/* /etc/fonts/conf.avail/
        ;;

    osx)
        ;;
esac
