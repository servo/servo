#!/bin/bash

set -e

case ${TRAVIS_OS_NAME} in
    linux)
        sudo add-apt-repository ppa:ubuntu-toolchain-r/test -y
        sudo apt-get update -q
        sudo apt-get install -qq --force-yes -y xserver-xorg-input-void xserver-xorg-video-dummy xpra
        sudo apt-get install -qq --force-yes -y gperf libXxf86vm-dev libstdc++6-4.7-dev
        echo ttf-mscorefonts-installer msttcorefonts/accepted-mscorefonts-eula select true | sudo debconf-set-selections
        sudo apt-get install ttf-mscorefonts-installer > /dev/null

        # install glfw
        git clone https://github.com/glfw/glfw.git
        cd glfw
        git checkout 3.0.3
        cmake -DCMAKE_C_FLAGS=-fPIC -DGLFW_BUILD_EXAMPLES=OFF -DGLFW_BUILD_TESTS=OFF -DGLFW_BUILD_DOCS=OFF .
        make
        sudo make install
        cd ..
        ;;

    osx)
        brew install pkg-config python glfw3
        pip install virtualenv
        ;;
esac
