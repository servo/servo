# The Servo Parallel Browser Engine Project

Servo is a prototype web browser engine written in the
[Rust](https://github.com/rust-lang/rust) language. It is currently developed on
64bit OS X, 64bit Linux, Android, and Gonk (Firefox OS).

Servo welcomes contribution from everyone.  See
[`CONTRIBUTING.md`](CONTRIBUTING.md) and [`HACKING_QUICKSTART.md`](HACKING_QUICKSTART.md)
for help getting started.

Visit the [Servo Project page](https://servo.org/) for news and guides.

## Prerequisites

On OS X (homebrew):

``` sh
brew install automake pkg-config python cmake
pip install virtualenv
```

On OS X (MacPorts):

``` sh
sudo port install python27 py27-virtualenv cmake
```

On OS X 10.11, you also have to install openssl:

``` sh
brew install openssl
brew link --force openssl
```

On Debian-based Linuxes:

``` sh
sudo apt-get install curl freeglut3-dev autoconf \
    libfreetype6-dev libgl1-mesa-dri libglib2.0-dev xorg-dev \
    gperf g++ build-essential cmake virtualenv python-pip \
    libssl-dev libbz2-dev libosmesa6-dev libxmu6 libxmu-dev \
    libglu1-mesa-dev libgles2-mesa-dev libegl1-mesa-dev
```
If you are on **Ubuntu 14.04** and encountered errors on installing these dependencies involving `libcheese`, see [#6158](https://github.com/servo/servo/issues/6158) for a workaround.

If `virtualenv` does not exist, try `python-virtualenv`.

On Fedora:

``` sh
sudo dnf install curl freeglut-devel libtool gcc-c++ libXi-devel \
    freetype-devel mesa-libGL-devel mesa-libEGL-devel glib2-devel libX11-devel libXrandr-devel gperf \
    fontconfig-devel cabextract ttmkfdir python python-virtualenv python-pip expat-devel \
    rpm-build openssl-devel cmake bzip2-devel libXcursor-devel libXmu-devel mesa-libOSMesa-devel
```

On Arch Linux:

``` sh
sudo pacman -S --needed base-devel git python2 python2-virtualenv python2-pip mesa cmake bzip2 libxmu glu
```

On Gentoo Linux:

```sh
sudo emerge net-misc/curl media-libs/freeglut \
    media-libs/freetype media-libs/mesa dev-util/gperf \
    dev-python/virtualenv dev-python/pip dev-libs/openssl \
    x11-libs/libXmu media-libs/glu x11-base/xorg-server
```

On Windows:

Download Python for Windows [here](https://www.python.org/downloads/release/python-2711/). This is
required for the SpiderMonkey build on Windows.

Install MSYS2 from [here](https://msys2.github.io/). After you have done so, open an MSYS shell
window and update the core libraries and install new packages:

```sh
update-core
pacman -Sy git mingw-w64-x86_64-toolchain mingw-w64-x86_64-freetype \
    mingw-w64-x86_64-icu mingw-w64-x86_64-nspr mingw-w64-x86_64-ca-certificates \
    mingw-w64-x86_64-expat mingw-w64-x86_64-cmake tar diffutils patch \
    patchutils make python2-setuptools
easy_install-2.7 pip virtualenv
```

Open a new MSYS shell window as Administrator and remove the Python binaries (they
are not compatible with our `mach` driver script yet, unfortunately):

```sh
cd /mingw64/bin
mv python2.exe python2-mingw64.exe
mv python2.7.exe python2.7-mingw64.exe
```

Now, open a MINGW64 (not MSYS!) shell window, and you should be able to build servo as usual!

Cross-compilation for Android:

Pre-installed Android tools are needed. See wiki for
[details](https://github.com/servo/servo/wiki/Building-for-Android)

## The Rust compiler

Servo's build system automatically downloads a Rust compiler to build itself.
This is normally a specific revision of Rust upstream, but sometimes has a
backported patch or two.
If you'd like to know which nightly build of Rust we use, see
[`rust-nightly-date`](https://github.com/servo/servo/blob/master/rust-nightly-date).

## Building

Servo is built with Cargo, the Rust package manager. We also use Mozilla's
Mach tools to orchestrate the build and other tasks.

### Normal build


To build Servo in development mode.  This is useful for development, but
the resulting binary is very slow.

``` sh
git clone https://github.com/servo/servo
cd servo
./mach build --dev
./mach run tests/html/about-mozilla.html
```

For benchmarking, performance testing, or
real-world use, add the `--release` flag to create an optimized build:

``` sh
./mach build --release
./mach run --release tests/html/about-mozilla.html
```

### Building for Android target

``` sh
git clone https://github.com/servo/servo
cd servo
ANDROID_TOOLCHAIN=/path/to/toolchain ANDROID_NDK=/path/to/ndk PATH=$PATH:/path/to/toolchain/bin ./mach build --android
cd ports/android
ANDROID_SDK=/path/to/sdk make install
```

Rather than setting the `ANDROID_*` environment variables every time, you can
also create a `.servobuild` file and then edit it to contain the correct paths
to the Android SDK/NDK tools:

```
cp servobuild.example .servobuild
# edit .servobuild
```

## Running

Use `./mach run [url]` to run Servo.


### Commandline Arguments

- `-p INTERVAL` turns on the profiler and dumps info to the console every
  `INTERVAL` seconds
- `-s SIZE` sets the tile size for painting; defaults to 512
- `-z` disables all graphical output; useful for running JS / layout tests

### Keyboard Shortcuts

- `Ctrl--` zooms out
- `Ctrl-=` zooms in
- `Backspace` goes backwards in the history
- `Shift-Backspace` goes forwards in the history
- `Esc` exits servo

## Developing

There are lots of mach commands you can use. You can list them with `./mach
--help`.


The generated documentation can be found on http://doc.servo.org/servo/index.html
