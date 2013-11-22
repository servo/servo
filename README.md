# The Servo Parallel Browser Project

### Note: Please use Linux or Mac OS only.

##Note getting started developing Rust


This page describes how to download and build the Rust compiler and associated tools and libraries from the current git sources.
If you're more interested in using Rust than in hacking on the Rust compiler,
you might prefer to install a released version, following the instructions in the tutorial.

Here is the link for setting up Rust on your Computer.
https://github.com/mozilla/rust/wiki/Note-getting-started-developing-Rust

##Note getting started developing Servo
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

=======
Generating a Binding for CanvasRenderingContext2D
===============================

- Add the implementation CanvasRenderingContext2D webidl file to (src/components/script/dom/bindings/codegen)[https://github.com/Aalhad/CanvasRenderingContext2DMozilla/tree/master/src/components/script/dom/bindings/codegen]
 
- Add an entry to (Bindings.conf)[https://github.com/Aalhad/CanvasRenderingContext2DMozilla/tree/master/src/components/script/dom/bindings/codegen/Bindings.conf] - if there's a addExternalInterface call that references the new name, remove it
 
- Create your implementation (canvasrenderingcontext2d.rs) in (src/components/script/dom)[https://github.com/Aalhad/CanvasRenderingContext2DMozilla/tree/master/src/components/script/dom] - you'll need a struct containing a Reflector at minimum, as well as implementation of the Reflectable and BindingObject traits. Look at (blob.rs)[https://github.com/mozilla/servo/blob/master/src/components/script/dom/blob.rs] for a nice, minimal reference.
 
- Add your implementation to (script.rc)[https://github.com/Aalhad/CanvasRenderingContext2DMozilla/tree/master/src/components/script/script.rc] to ensure it is compiled.

- Build, and fix up any compile errors (such as missing methods or incorrect argument types in your new implementation).


