# The Servo Parallel Browser Project

Servo is a prototype web browser engine written in the [Rust](https://github.com/mozilla/rust)
language. It is currently developed on OS X and Linux.

## Prerequisites

On OS X (homebrew):

    brew install https://raw.github.com/Homebrew/homebrew-versions/master/autoconf213.rb
    brew install automake libtool
    brew install pkg-config

Note, there is an [issue][] on homebrew which requires the following manual
configuration as well:

    sudo sh -c 'echo /usr/local/share/aclocal >> /usr/share/aclocal/dirlist'

On OS X (MacPorts):

    sudo port install autoconf213
    
On Debian-based Linuxes:

    sudo apt-get install autoconf2.13 freeglut3-dev libtool

Servo builds its own copy of Rust, so there is no need to provide a Rust
compiler.

## Building

    git clone git://github.com/mozilla/servo.git
    cd servo
    mkdir -p build && cd build
    ../configure
    make && make check
    ./servo ../src/test/about-mozilla.html

[issue]: https://github.com/mxcl/homebrew/issues/5117
