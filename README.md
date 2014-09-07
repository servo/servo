The Servo Parallel Browser Project

Servo is a prototype web browser engine written in the [Rust](https://github.com/mozilla/rust)
language. It is currently developed on 64bit OS X and 64bit Linux.

Servo welcomes contribution from everyone.  See
[`CONTRIBUTING.md`](CONTRIBUTING.md) for help getting started.

## Prerequisites

On OS X (homebrew):

``` sh
brew install https://raw.github.com/Homebrew/homebrew-versions/master/autoconf213.rb
brew install automake libtool pkg-config python
pip install virtualenv
```

On OS X (MacPorts):

``` sh
sudo port install autoconf213 python27 py27-virtualenv
```

On Debian-based Linuxes:

``` sh
sudo apt-get install autoconf2.13 curl freeglut3-dev libtool \
    libfreetype6-dev libgl1-mesa-dri libglib2.0-dev xorg-dev \
    msttcorefonts gperf g++ automake cmake python-virtualenv \
    libssl-dev
```

On Fedora:

``` sh
sudo yum install autoconf213 curl freeglut-devel libtool gcc-c++ libXi-devel \
    freetype-devel mesa-libGL-devel glib2-devel libX11-devel libXrandr-devel gperf \
    fontconfig-devel cabextract ttmkfdir python python-virtualenv expat-devel \
    rpm-build openssl-devel
pushd .
cd /tmp
wget http://corefonts.sourceforge.net/msttcorefonts-2.5-1.spec
rpmbuild -bb msttcorefonts-2.5-1.spec
sudo yum install $HOME/rpmbuild/RPMS/noarch/msttcorefonts-2.5-1.noarch.rpm
popd
```

On Arch Linux:

``` sh
sudo pacman -S base-devel autoconf2.13 git gperf python2 \
    python2-virtualenv mesa libxrandr libxi libgl glu ttf-font
```

Note: autoconf 2.13 is required for SpiderMonkey; the autoconf project did not
preserve backwards compatibility after version 2.13, and changing the Firefox
build to work with a newer version is not considered a good use of developers'
time.

Cross-compilation for Android:

Basically, pre-installed Android tools are needed.
See wiki for [details](https://github.com/mozilla/servo/wiki/Building-for-Android)

## The Rust compiler

Servo builds its own copy of Rust, so there is no need to provide a Rust
compiler.
If you'd like to know the snapshot revision of Rust which we use, see `./rust-snapshot-hash`.

## Building

Servo cannot be built in-tree; you must create a directory in which to run
configure and make and place the build artifacts.

``` sh
git clone https://github.com/servo/servo.git
cd servo
./mach build
./mach run tests/html/about-mozilla.html
```

### Building for Android target

``` sh
git clone https://github.com/servo/servo.git
cd servo
ANDROID_TOOLCHAIN=/path/to/toolchain ANDROID_NDK=/path/to/ndk PATH=$PATH:/path/to/toolchain/bin ./mach build --target arm-linux-androideabi
cd ports/android
ANDROID_NDK=/path/to/ndk ANDROID_SDK=/path/to/sdk make
ANDROID_SDK=/path/to/sdk make install
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

There are lots of mach commands you can use. You can list them with `./mach --help`.
