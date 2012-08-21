## The Servo Parallel Browser Project

Servo is a web browser engine written in the Rust language. It is
currently developed on OS X and Linux.

Note: Servo requires a bleeding-edge version of Rust. Sometimes this
means working off of the Rust _master_ branch; sometimes this means
the _incoming_ branch. Because Rust is still undergoing major changes
the Servo build is very often broken. Somebody in #servo or #rust
will usually know what magic is required to make Servo build on any
given day. Good luck!

### Prerequisites

On OS X (homebrew):

    brew install https://raw.github.com/Homebrew/homebrew-versions/master/autoconf213.rb
    brew install sdl cairo

On OS X (MacPorts):

    sudo port install autoconf213 libsdl cairo +x11 +quartz
    
On Debian-based Linuxes:

    sudo apt-get install libsdl1.2-dev libcairo2-dev libpango1.0-dev autoconf2.13 freeglut3-dev

### Building

    git clone git://github.com/mozilla/servo.git
    cd servo
    git submodule init
    git submodule update
    ./autogen.sh
    mkdir -p build && cd build
    ../configure
    make check && make
    ./servo ../src/test/test.html


### Build Workarounds

#### MacPorts

Currently, the Makefile for the rust-azure submodule has a hardcoded
library path that assumes cairo has been installed with homebrew. If
you have installed with MacPorts, you will need to change the library
path to cairo. The following command should apply a patch with the fix:

    cd src/rust-azure && git diff 1e811d44^1 1e811d44 | patch -p1

This problem should go away once Issue #40 is fixed, and an
externally-built cairo is no longer needed.
