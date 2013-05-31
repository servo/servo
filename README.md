The Servo Parallel Browser Project

Servo is a prototype web browser engine written in the [Rust](https://github.com/mozilla/rust)
language. It is currently developed on 64bit OS X and 64bit Linux.

## Prerequisites

On OS X (homebrew):

``` sh
brew install https://raw.github.com/Homebrew/homebrew-versions/master/autoconf213.rb
brew install automake libtool
brew install pkg-config
```

On OS X (MacPorts):

``` sh
sudo port install autoconf213
```
    
On Debian-based Linuxes:

``` sh
sudo apt-get install autoconf2.13 curl freeglut3-dev libtool libfreetype6-dev libfontconfig1-dev libgl1-mesa-dri libglib2.0-dev
```

Servo builds its own copy of Rust, so there is no need to provide a Rust
compiler.

## Building

``` sh
git clone git://github.com/mozilla/servo.git
cd servo
mkdir -p build && cd build
../configure
make && make check
./servo ../src/test/html/about-mozilla.html
```

[issue]: https://github.com/mxcl/homebrew/issues/5117
