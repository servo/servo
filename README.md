## The Servo Parallel Browser Project

I currently build on OS X and Linux.

### Prerequisites

On OS X:

    brew install sdl cairo

On Debian-based Linuxes:

    sudo apt-get install sdl cairo

### Building

    git clone git://github.com/mozilla/servo.git
    cd servo
    git submodule init
    git submodule update
    ./autogen.sh
    mkdir build && cd build
    ../configure
    make check && make
    ./servo ../test.html
