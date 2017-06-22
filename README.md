# The Servo Parallel Browser Engine Project

[![Linux Build Status](https://img.shields.io/travis/servo/servo/master.svg?label=Linux%20build)](https://travis-ci.org/servo/servo)  [![Windows Build Status](https://img.shields.io/appveyor/ci/servo/servo/master.svg?label=Windows%20build)](https://ci.appveyor.com/project/servo/servo/branch/master)  [![Changelog #228](https://img.shields.io/badge/changelog-%23228-9E978E.svg)](https://changelog.com/podcast/228)

Servo is a prototype web browser engine written in the
[Rust](https://github.com/rust-lang/rust) language. It is currently developed on
64bit OS X, 64bit Linux, and Android.

Servo welcomes contribution from everyone.  See
[`CONTRIBUTING.md`](CONTRIBUTING.md) and [`HACKING_QUICKSTART.md`](docs/HACKING_QUICKSTART.md)
for help getting started.

Visit the [Servo Project page](https://servo.org/) for news and guides.

## Setting up your environment

Please select your operating system:
* [OS X](#os-x)
* [Debian-based Linuxes](#on-debian-based-linuxes)
* [Fedora](#on-fedora)
* [Arch Linux](#on-arch-linux)
* [openSUSE](#on-opensuse-linux)
* [Gentoo Linux](#on-gentoo-linux)
* [Microsoft Windows](#on-windows-msvc)
* [Android](#cross-compilation-for-android)

#### OS X
#### On OS X (homebrew)

``` sh
brew install automake pkg-config python cmake yasm
pip install virtualenv
```
#### On OS X (MacPorts)

``` sh
sudo port install python27 py27-virtualenv cmake yasm
```
#### On OS X >= 10.11 (El Capitan), you also have to install OpenSSL

``` sh
brew install openssl

export OPENSSL_INCLUDE_DIR="$(brew --prefix openssl)/include"
export OPENSSL_LIB_DIR="$(brew --prefix openssl)/lib"

./mach build ...
```

If you've already partially compiled servo but forgot to do this step, run `./mach clean`, set the shell variables, and recompile.

#### On Debian-based Linuxes

``` sh
sudo apt install git curl freeglut3-dev autoconf \
    libfreetype6-dev libgl1-mesa-dri libglib2.0-dev xorg-dev \
    gperf g++ build-essential cmake virtualenv python-pip \
    libssl1.0-dev libbz2-dev libosmesa6-dev libxmu6 libxmu-dev \
    libglu1-mesa-dev libgles2-mesa-dev libegl1-mesa-dev libdbus-1-dev
```

If you using a version prior to **Ubuntu 17.04** or **Debian Sid**, replace `libssl1.0-dev` with `libssl-dev`.

If you are on **Ubuntu 14.04** and encountered errors on installing these dependencies involving `libcheese`, see [#6158](https://github.com/servo/servo/issues/6158) for a workaround.

If `virtualenv` does not exist, try `python-virtualenv`.

#### On Fedora

``` sh
sudo dnf install curl freeglut-devel libtool gcc-c++ libXi-devel \
    freetype-devel mesa-libGL-devel mesa-libEGL-devel glib2-devel libX11-devel libXrandr-devel gperf \
    fontconfig-devel cabextract ttmkfdir python python-virtualenv python-pip expat-devel \
    rpm-build openssl-devel cmake bzip2-devel libXcursor-devel libXmu-devel mesa-libOSMesa-devel \
    dbus-devel ncurses-devel
```
#### On openSUSE Linux
``` sh
sudo zypper install libX11-devel libexpat-devel libbz2-devel Mesa-libEGL-devel Mesa-libGL-devel cabextract cmake \
    dbus-1-devel fontconfig-devel freetype-devel gcc-c++ git glib2-devel gperf \
    harfbuzz-devel libOSMesa-devel libXcursor-devel libXi-devel libXmu-devel libXrandr-devel libopenssl-devel \
    python-pip python-virtualenv rpm-build glu-devel
```
#### On Arch Linux

``` sh
sudo pacman -S --needed base-devel git python2 python2-virtualenv python2-pip mesa cmake bzip2 libxmu glu pkg-config
```
#### On Gentoo Linux

```sh
sudo emerge net-misc/curl media-libs/freeglut \
    media-libs/freetype media-libs/mesa dev-util/gperf \
    dev-python/virtualenv dev-python/pip dev-libs/openssl \
    x11-libs/libXmu media-libs/glu x11-base/xorg-server
```
#### On Windows (MSVC)

1. Install Python for Windows (https://www.python.org/downloads/release/python-2711/). The Windows x86-64 MSI installer is fine.
You should change the installation to install the "Add python.exe to Path" feature.

2. Install virtualenv.

 In a normal Windows Shell (cmd.exe or "Command Prompt" from the start menu), do:
 ```
pip install virtualenv
```
 If this does not work, you may need to reboot for the changed PATH settings (by the python installer) to take effect.

3. Install Git for Windows (https://git-scm.com/download/win). DO allow it to add git.exe to the PATH (default
settings for the installer are fine).

4. Install Visual Studio 2015 Community Edition (https://www.visualstudio.com/). You MUST add "Visual C++" to the
list of installed components. It is not on by default.

#### Cross-compilation for Android

Pre-installed Android tools are needed. See wiki for
[details](https://github.com/servo/servo/wiki/Building-for-Android)

## The Rust compiler

Servo's build system automatically downloads a Rust compiler to build itself.
This is normally a specific revision of Rust upstream, but sometimes has a
backported patch or two.
If you'd like to know which nightly build of Rust we use, see
[`rust-commit-hash`](https://github.com/servo/servo/blob/master/rust-commit-hash).

## Building

Servo is built with [Cargo](https://crates.io/), the Rust package manager. We also use Mozilla's
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

Or on Windows MSVC, in a normal Command Prompt (cmd.exe):
``` cmd
git clone https://github.com/servo/servo
cd servo
mach.bat build --dev
```

For benchmarking, performance testing, or
real-world use, add the `--release` flag to create an optimized build:

``` sh
./mach build --release
./mach run --release tests/html/about-mozilla.html
```

### Checking for build errors, without building

If you’re making changes to one crate that cause build errors in another crate,
consider this instead of a full build:

```sh
./mach check
```

It will run `cargo check`, which runs the analysis phase of the compiler
(and so shows build errors if any) but skips the code generation phase.
This can be a lot faster than a full build,
though of course it doesn’t produce a binary you can run.

### Building for Android target

``` sh
git clone https://github.com/servo/servo
cd servo

export ANDROID_SDK="/path/to/sdk"
export ANDROID_NDK="/path/to/ndk"
export ANDROID_TOOLCHAIN="/path/to/toolchain"
export PATH="$PATH:/path/to/toolchain/bin"

./mach build --release --android
./mach package --release --android
```

Rather than setting the `ANDROID_*` environment variables every time, you can
also create a `.servobuild` file and then edit it to contain the correct paths
to the Android SDK/NDK tools:

```
cp servobuild.example .servobuild
# edit .servobuild
```

## Running

Use `./mach run [url]` to run Servo. Like so,

``` shell
./mach run https://www.google.com
```

Also, don't miss the info on the [browserhtml page](https://github.com/browserhtml/browserhtml) on how to run the Browser.html
full tech demo (it provides a more browser-like experience than just browsing a single
URL with servo).


### Commandline Arguments

- `-p INTERVAL` turns on the profiler and dumps info to the console every
  `INTERVAL` seconds
- `-s SIZE` sets the tile size for painting; defaults to 512
- `-z` disables all graphical output; useful for running JS / layout tests
- `-Z help` displays useful output to debug servo

### Keyboard Shortcuts

- `Ctrl`+`-` zooms out
- `Ctrl`+`=` zooms in
- `Alt`+`left arrow` goes backwards in the history
- `Alt`+`right arrow` goes forwards in the history
- `Esc` exits servo

## Developing

There are lots of mach commands you can use. You can list them with `./mach
--help`.


The generated documentation can be found on http://doc.servo.org/servo/index.html
