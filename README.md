## The Servo Parallel Browser Project

I currently build on OS X and Linux.

### Prerequisites

On OS X:

    brew install https://raw.github.com/Homebrew/homebrew-versions/master/autoconf213.rb
    brew install sdl cairo

On Debian-based Linuxes:

    sudo apt-get install libsdl1.2-dev libcairo2-dev libpango1.0-dev

### Building

    git clone git://github.com/mozilla/servo.git
    cd servo
    git submodule init
    git submodule update
    ./autogen.sh
    mkdir build && cd build
    ../configure
    make check && make
    ./servo ../src/test/test.html
