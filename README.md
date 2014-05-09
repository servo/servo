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
    libfreetype6-dev libgl1-mesa-dri libglib2.0-dev xorg-dev \
    msttcorefonts gperf g++ automake cmake
```

On Fedora Core:

``` sh
sudo yum install autoconf213 curl freeglut-devel libtool \
    freetype-devel mesa-libGL-devel glib2-devel libX11-devel \
    gperf gcc-c++ rpm-build cabextract ttmkfdir
pushd .
cd /tmp
wget http://corefonts.sourceforge.net/msttcorefonts-2.5-1.spec
rpmbuild -bb msttcorefonts-2.5-1.spec
sudo yum install $HOME/rpmbuild/RPMS/noarch/msttcorefonts-2.5-1.noarch.rpm 
popd
```

Cross-compilation for Android:

Basically, pre-installed Android tools are needed.
See wiki for [details](https://github.com/mozilla/servo/wiki/Building-for-Android)

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
../configure --target=arm-linux-androideabi --android-cross-path=<Android toolchain path> --android-ndk-path=<Android NDK path> --android-sdk-path=<Android SDK path>
make
(or make package)

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
- `make clean-servo` - only cleans Servo itself (code in `src/components`)
- `make clean-DEP` - cleans the dependency `DEP`. e.g. `make clean-rust-opengles`
- `make bindings` - generate the Rust WebIDL bindings
- `make DEP` - builds only the specified dependency. e.g. `make rust-opengles`
- `make check-DEP` - build and run tests for specified dependency
- `make package` - build and make app package for specific OS. e.g. apk file of Android

The `make check-*` targets for running tests are listed [here](https://github.com/mozilla/servo/wiki/Testing)
