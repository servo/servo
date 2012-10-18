# The Servo Parallel Browser Project

Servo is a prototype web browser engine written in the [Rust]
language. It is currently developed on OS X and Linux.

**Note:** Servo always requires a specific version of Rust - building
against a released version of Rust will not work, nor will the Rust
'master' branch. The commit below will *probably* work. If it does not
then the topic in #servo might know better.

* Last known-good Rust commit: 57b4d10ff652d3beddae64782c882a07822bac3c

[rust]: http://www.rust-lang.org

## Prerequisites

First, you need the Rust compiler, built from the exact commit listed
above.

On OS X (homebrew):

    brew install https://raw.github.com/Homebrew/homebrew-versions/master/autoconf213.rb
    brew install cairo

On OS X (MacPorts):

    sudo port install autoconf213 cairo +x11 +quartz
    
On Debian-based Linuxes:

    sudo apt-get install libcairo2-dev libpango1.0-dev autoconf2.13 freeglut3-dev

## Building

    git clone git://github.com/mozilla/servo.git
    cd servo
    mkdir -p build && cd build
    ../configure
    make check-servo && make
    ./servo ../src/test/hello.html

If `rustc` is not installed then add `RUSTC=/path/to/rustc` to your
`make` commands.


## Build Workarounds

### MacPorts

Currently, the Makefile for the `rust-azure` submodule has hardcoded
library paths that assumes cairo has been installed with homebrew or
MacPorts. If you have installed cairo via another methods or a
different version, you will need to change the library path.

This problem should go away once Issue #40 is fixed, and an
externally-built cairo is no longer needed.
