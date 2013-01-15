# The Servo Parallel Browser Project

Servo is a prototype web browser engine written in the [Rust]
language. It is currently developed on OS X and Linux.

## Prerequisites

On OS X (homebrew):

    brew install https://raw.github.com/Homebrew/homebrew-versions/master/autoconf213.rb
    brew install cairo

On OS X (MacPorts):

    sudo port install autoconf213 cairo +x11 +quartz
    
On Debian-based Linuxes:

    sudo apt-get install libcairo2-dev libpango1.0-dev autoconf2.13 freeglut3-dev libtool

Servo builds its own copy of Rust, so there is no need to provide a Rust
compiler.

## Building

    git clone git://github.com/mozilla/servo.git
    cd servo
    mkdir -p build && cd build
    ../configure
    make check-servo && make
    ./servo ../src/test/hello.html


## Build Workarounds

### MacPorts

Currently, the Makefile for the `rust-azure` submodule has hardcoded
library paths that assumes cairo has been installed with homebrew or
MacPorts. If you have installed cairo via another methods or a
different version, you will need to change the library path.

This problem should go away once Issue #40 is fixed, and an
externally-built cairo is no longer needed.
