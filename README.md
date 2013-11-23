The Servo Parallel Browser Project

Servo is a prototype web browser engine written in the [Rust](https://github.com/mozilla/rust)
language. It is currently developed on 64bit OS X and 64bit Linux.

Servo welcomes contribution from everyone.  See
[`CONTRIBUTING.md`](CONTRIBUTING.md) for help getting started.

## Prerequisites

On OS X (homebrew):

``` sh
brew install https://raw.github.com/Homebrew/homebrew-versions/master/autoconf213.rb
brew install automake libtool pkg-config

    Explore
    Gist
    Blog
    Help

    Aalhad

    165
    1,287
    199

public mozilla / servo

You are editing a file in a project you do not have write access to. We are forking this project for you (if one does not yet exist) to write your proposed changes to. Submitting a change to this file will write it to a new branch in your fork so you can send a pull request.
servo /

or cancel

1
2
3
4
5
6
7
8
9
10
11
12
13
14
15
16
17
18
19
20
21
22
23
24
25
26
27
28
29
30
31
32
33
34
35
36
37
38
39
40
41
42
43
The Servo Parallel Browser Project
Servo is a prototype web browser engine written in the [Rust](https://github.com/mozilla/rust)
language. It is currently developed on 64bit OS X and 64bit Linux.
Servo welcomes contribution from everyone.  See
[`CONTRIBUTING.md`](CONTRIBUTING.md) for help getting started.
## Prerequisites
On OS X (homebrew):
``` sh
brew install https://raw.github.com/Homebrew/homebrew-versions/master/autoconf213.rb
brew install automake libtool pkg-config
```
On OS X (MacPorts):
``` sh
sudo port install autoconf213
```
    
On Debian-based Linuxes:
``` sh
sudo apt-get install autoconf2.13 curl freeglut3-dev libtool \
    libfreetype6-dev libfontconfig1-dev libgl1-mesa-dri libglib2.0-dev \
    xorg-dev msttcorefonts
```
On Debian-based Linuxes (cross-compilation for Android):
``` sh
sudo apt-get install autoconf2.13 curl libtool ia32-libs
```
And it needs pre-installed Android tools.
See wiki for [details](https://github.com/mozilla/servo/wiki/Doc-building-for-android)
Servo builds its own copy of Rust, so there is no need to provide a Rust
compiler.
Commit summary: Extended description: (optional)
Aalhad aalhad_22@yahoo.com

    Status
    API
    Training
    Shop
    Blog
    About

    © 2013 GitHub, Inc.
    Terms
    Privacy
    Security
    Contact


```

On OS X (MacPorts):

``` sh
sudo port install autoconf213
```
    
On Debian-based Linuxes:

``` sh
sudo apt-get install autoconf2.13 curl freeglut3-dev libtool \
    libfreetype6-dev libfontconfig1-dev libgl1-mesa-dri libglib2.0-dev \
    xorg-dev msttcorefonts
```

On Debian-based Linuxes (cross-compilation for Android):

``` sh
sudo apt-get install autoconf2.13 curl libtool ia32-libs
```
And it needs pre-installed Android tools.
See wiki for [details](https://github.com/mozilla/servo/wiki/Doc-building-for-android)


Servo builds its own copy of Rust, so there is no need to provide a Rust
compiler.

## Building

Servo cannot be built in-tree; you must create a directory in which to run
configure and make and place the build artifacts.

``` sh
git clone https://github.com/mozilla/servo.git
cd servo
mkdir -p build && cd build
../configure
make && make check
./servo ../src/test/html/about-mozilla.html
```

###Building for Android target

``` sh
git clone https://github.com/mozilla/servo.git
cd servo
mkdir -p build && cd build
../configure --target-triples=arm-linux-androideabi --android-cross-path=<Android toolchain path> --android-ndk-path=<Android NDK path> --android-sdk-path=<Android SDK path>
make
```

## Running

### Commandline Arguments

- `-p INTERVAL` turns on the profiler and dumps info to the console every
  `INTERVAL` seconds
- `-s SIZE` sets the tile size for rendering; defaults to 512
- `-z` disables all graphical output; useful for running JS / layout tests

### Keyboard Shortcuts

- `Ctrl-L` opens a dialog to browse to a new URL (Mac only currently)
- `Ctrl--` zooms out
- `Ctrl-=` zooms in
- `Backspace` goes backwards in the history
- `Shift-Backspace` goes forwards in the history
- `Esc` exits servo

## Developing

There are lots of make targets you can use:

- `make clean` - cleans Servo and its dependencies, but not Rust
- `make clean-rust` - cleans Rust
- `make clean-servo` - only cleans Servo itself (code in `src/components`
- `make clean-DEP` - cleans the dependency `DEP`. e.g. `make clean-rust-opengles`
- `make bindings` - generate the Rust WebIDL bindings
- `make DEP` - builds only the specified dependency. e.g. `make rust-opengles`
- `make check-DEP` - build and run tests for specified dependency

