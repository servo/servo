# The Servo Parallel Browser Engine Project

[![Linux Build Status](https://img.shields.io/travis/servo/servo/master.svg?label=Linux%20build)](https://travis-ci.org/servo/servo)  [![Windows Build Status](https://img.shields.io/appveyor/ci/servo/servo/master.svg?label=Windows%20build)](https://ci.appveyor.com/project/servo/servo/branch/master)  [![Changelog #228](https://img.shields.io/badge/changelog-%23228-9E978E.svg)](https://changelog.com/podcast/228)

Servo is a prototype web browser engine written in the
[Rust](https://github.com/rust-lang/rust) language. It is currently developed on
64-bit macOS, 64-bit Linux, 64-bit Windows, and Android.

Servo welcomes contribution from everyone.  See
[`CONTRIBUTING.md`](CONTRIBUTING.md) and [`HACKING_QUICKSTART.md`](docs/HACKING_QUICKSTART.md)
for help getting started.

Visit the [Servo Project page](https://servo.org/) for news and guides.

## Setting up your environment

### Rustup.rs

Building servo requires [rustup](https://rustup.rs/), version 1.8.0 or more recent.
If you have an older version, run `rustup self update`.

To install on Windows, download and run [`rustup-init.exe`](https://win.rustup.rs/)
then follow the onscreen instructions.

To install on other systems, run:

```sh
curl https://sh.rustup.rs -sSf | sh
```

This will also download the current stable version of Rust, which Servo won’t use.
To skip that step, run instead:

```
curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain none
```

See also [Other installation methods](
https://github.com/rust-lang-nursery/rustup.rs/#other-installation-methods)

### Other dependencies

Please select your operating system:
* [macOS](#macos)
* [Debian-based Linuxes](#on-debian-based-linuxes)
* [Fedora](#on-fedora)
* [Arch Linux](#on-arch-linux)
* [openSUSE](#on-opensuse-linux)
* [Gentoo Linux](#on-gentoo-linux)
* [Microsoft Windows](#on-windows-msvc)
* [Android](#cross-compilation-for-android)

#### macOS
##### On macOS (homebrew)

``` sh
brew bundle install --file=etc/taskcluster/macos/Brewfile
pip install virtualenv
```
##### On macOS (MacPorts)

``` sh
sudo port install python27 py27-virtualenv cmake yasm llvm
```
##### On macOS >= 10.11 (El Capitan), you also have to install OpenSSL

``` sh
brew install openssl

export OPENSSL_INCLUDE_DIR="$(brew --prefix openssl)/include"
export OPENSSL_LIB_DIR="$(brew --prefix openssl)/lib"

./mach build ...
```

If you've already partially compiled servo but forgot to do this step, run `./mach clean`, set the shell variables, and recompile.

#### On Debian-based Linuxes

Please run `./mach bootstrap`.

If this doesn't work, file a bug, and, run the commands below:

``` sh
sudo apt install git curl autoconf libx11-dev \
    libfreetype6-dev libgl1-mesa-dri libglib2.0-dev xorg-dev \
    gperf g++ build-essential cmake virtualenv python-pip \
    libssl1.0-dev libbz2-dev libosmesa6-dev libxmu6 libxmu-dev \
    libglu1-mesa-dev libgles2-mesa-dev libegl1-mesa-dev libdbus-1-dev \
    libharfbuzz-dev ccache clang \
    libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev libgstreamer-plugins-bad1.0-dev autoconf2.13
```

If you using a version prior to **Ubuntu 17.04**, **Linux Mint 19** or **Debian Sid**, replace `libssl1.0-dev` with `libssl-dev`.
Additionally, you'll need a local copy of GStreamer with a version later than 12.0. You can place it in `support/linux/gstreamer/gstreamer`, or run `./mach bootstrap-gstreamer` to set it up.

If you are using **Ubuntu 16.04** or **Linux Mint 18.&#42;** run `export HARFBUZZ_SYS_NO_PKG_CONFIG=1` before building to avoid an error with harfbuzz.

If you get an undefined symbol error on `gst_player_get_config` try removing `gir1.2-gst-plugins-bad-1.0` and all old versions of clang, see [#22016](https://github.com/servo/servo/issues/22016)

If you are on **Ubuntu 14.04** and encountered errors on installing these dependencies involving `libcheese`, see [#6158](https://github.com/servo/servo/issues/6158) for a workaround. You may also need to install gcc 4.9, clang 4.0, and cmake 3.2:

<details>
gcc 4.9:

```sh
sudo add-apt-repository ppa:ubuntu-toolchain-r/test
sudo apt-get update
sudo apt-get install gcc-4.9 g++-4.9
sudo update-alternatives --install /usr/bin/gcc gcc /usr/bin/gcc-4.9 60 --slave /usr/bin/g++ g++ /usr/bin/g++-4.9
```

clang 4.0:

```sh
wget -O - https://apt.llvm.org/llvm-snapshot.gpg.key | sudo apt-key add -
sudo apt-add-repository "deb http://apt.llvm.org/trusty/ llvm-toolchain-trusty-4.0 main"
sudo apt-get update
sudo apt-get install -y clang-4.0
```

cmake 3.2:

```sh
sudo apt-get install software-properties-common
sudo add-apt-repository ppa:george-edison55/cmake-3.x
sudo apt-get update
sudo apt-get install cmake
```

</details>

If `virtualenv` does not exist, try `python-virtualenv`.

#### On Fedora

Please run `./mach bootstrap`.

If this doesn't work, file a bug, and, run the commands below:

``` sh
sudo dnf install curl libtool gcc-c++ libXi-devel \
    freetype-devel mesa-libGL-devel mesa-libEGL-devel glib2-devel libX11-devel libXrandr-devel gperf \
    fontconfig-devel cabextract ttmkfdir python2 python2-virtualenv python2-pip expat-devel \
    rpm-build openssl-devel cmake bzip2-devel libXcursor-devel libXmu-devel mesa-libOSMesa-devel \
    dbus-devel ncurses-devel harfbuzz-devel ccache mesa-libGLU-devel clang clang-libs gstreamer1-devel \
    gstreamer1-plugins-base-devel gstreamer1-plugins-bad-free-devel autoconf213
```
#### On CentOS


Please run `./mach bootstrap`.

If this doesn't work, file a bug, and, run the commands below:

``` sh
sudo yum install curl libtool gcc-c++ libXi-devel \
    freetype-devel mesa-libGL-devel mesa-libEGL-devel glib2-devel libX11-devel libXrandr-devel gperf \
    fontconfig-devel cabextract ttmkfdir python python-virtualenv python-pip expat-devel \
    rpm-build openssl-devel cmake3 bzip2-devel libXcursor-devel libXmu-devel mesa-libOSMesa-devel \
    dbus-devel ncurses-devel python34 harfbuzz-devel ccache clang clang-libs llvm-toolset-7
```

Build inside `llvm-toolset` and `devtoolset`:

```sh
scl enable devtoolset-7 llvm-toolset-7 bash
```

with the following environmental variables set:

```sh
export CMAKE=cmake3
export LIBCLANG_PATH=/opt/rh/llvm-toolset-7/root/usr/lib64
```
#### On openSUSE Linux
``` sh
sudo zypper install libX11-devel libexpat-devel libbz2-devel Mesa-libEGL-devel Mesa-libGL-devel cabextract cmake \
    dbus-1-devel fontconfig-devel freetype-devel gcc-c++ git glib2-devel gperf \
    harfbuzz-devel libOSMesa-devel libXcursor-devel libXi-devel libXmu-devel libXrandr-devel libopenssl-devel \
    python-pip python-virtualenv rpm-build glu-devel ccache llvm-clang libclang
```
#### On Arch Linux

``` sh
sudo pacman -S --needed base-devel git python2 python2-virtualenv python2-pip mesa cmake bzip2 libxmu glu \
    pkg-config ttf-fira-sans harfbuzz ccache clang autoconf2.13 gstreamer gstreamer-vaapi
```
#### On Gentoo Linux

```sh
sudo emerge net-misc/curl \
    media-libs/freetype media-libs/mesa dev-util/gperf \
    dev-python/virtualenv dev-python/pip dev-libs/openssl \
    media-libs/harfbuzz dev-util/ccache \
    x11-libs/libXmu media-libs/glu x11-base/xorg-server sys-devel/clang \
    media-libs/gstreamer media-libs/gst-plugins-bad media-libs/gst-plugins-base
```

with the following environment variable set:

```sh
export LIBCLANG_PATH="/usr/lib64/llvm/*/lib64"
```
#### On Windows (MSVC)

1. Install Python for Windows (https://www.python.org/downloads/release/python-2714/). The Windows x86-64 MSI installer is fine.
You should change the installation to install the "Add python.exe to Path" feature.

2. Install virtualenv.

 In a normal Windows Shell (cmd.exe or "Command Prompt" from the start menu), do:
 ```
pip install virtualenv
```
 If this does not work, you may need to reboot for the changed PATH settings (by the python installer) to take effect.

3. Install the most recent [GStreamer](https://gstreamer.freedesktop.org/data/pkg/windows/) development package following [these instructions](https://github.com/sdroege/gstreamer-rs#gstreamer-binaries-1). You will also need to add `C:\gstreamer\1.0\x86_64\bin` to your `LIB` environment variable.

4. Install Git for Windows (https://git-scm.com/download/win). DO allow it to add git.exe to the PATH (default
settings for the installer are fine).

5. Install Visual Studio Community 2017 (https://www.visualstudio.com/vs/community/). You MUST add "Visual C++" to the
list of installed components. It is not on by default. Visual Studio 2017 MUST installed to the default location or mach.bat will not find it.

##### [Optional] Install LLVM for faster link times

You may experience much faster builds on Windows by following these steps. (Related Rust issue: https://github.com/rust-lang/rust/issues/37543)

1. Download the latest version of LLVM (https://releases.llvm.org/).
2. Run the installer and choose to add LLVM to the system PATH.
3. Add the following to your Cargo config (Found at `%USERPROFILE%\.cargo\config`). You may need to change the triple to match your environment.

```
[target.x86_64-pc-windows-msvc]
linker = "lld-link.exe"
```

##### Troubleshooting a Windows environment

> If you encountered errors with the environment above, do the following for a workaround:
> 1.  Download and install [Build Tools for Visual Studio 2017](https://www.visualstudio.com/thank-you-downloading-visual-studio/?sku=BuildTools&rel=15)
> 2.  Install `python2.7 x86-x64` and `virtualenv`
> 3.  Run `mach.bat build -d`.

>If you have troubles with `x64 type` prompt as `mach.bat` set by default:
> 1. You may need to choose and launch the type manually, such as `x86_x64 Cross Tools Command Prompt for VS 2017` in the Windows menu.)
> 2. `cd to/the/path/servo`
> 3. `python mach build -d`

#### Cross-compilation for Android

Run `./mach bootstrap-android --build` to get Android-specific tools. See wiki for
[details](https://github.com/servo/servo/wiki/Building-for-Android).

## The Rust compiler

Servo's build system uses rustup.rs to automatically download a Rust compiler.
This is a specific version of Rust Nightly determined by the
[`rust-toolchain`](https://github.com/servo/servo/blob/master/rust-toolchain) file.

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

**Note:** `mach build ` will build both `servo` and `libsimpleservo`. To make compilation a bit faster, it's possible to only compile the servo binary: `./mach build --dev -p servo`.

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

For ARM (`armv7-linux-androideabi`, most phones):

``` sh
./mach build --release --android
./mach package --release --android
```

For x86 (typically for the emulator):

```sh
./mach build --release --target i686-linux-android
./mach package --release --target i686-linux-android
```

## Running

Run Servo with the command:

```sh
./servo [url] [arguments] # if you run with nightly build
./mach run [url] [arguments] # if you run with mach

# For example
./mach run https://www.google.com
```

### Commandline Arguments

- `-p INTERVAL` turns on the profiler and dumps info to the console every
  `INTERVAL` seconds
- `-s SIZE` sets the tile size for painting; defaults to 512
- `-z` disables all graphical output; useful for running JS / layout tests
- `-Z help` displays useful output to debug servo

### Keyboard Shortcuts

- `Ctrl`+`L` opens URL prompt (`Cmd`+`L` on Mac)
- `Ctrl`+`R` reloads current page (`Cmd`+`R` on Mac)
- `Ctrl`+`-` zooms out (`Cmd`+`-` on Mac)
- `Ctrl`+`=` zooms in (`Cmd`+`=` on Mac)
- `Alt`+`left arrow` goes backwards in the history (`Cmd`+`left arrow` on Mac)
- `Alt`+`right arrow` goes forwards in the history (`Cmd`+`right arrow` on Mac)
- `Esc` or `Ctrl`+`Q` exits Servo (`Cmd`+`Q` on Mac)

## Developing

There are lots of mach commands you can use. You can list them with `./mach
--help`.


The generated documentation can be found on https://doc.servo.org/servo/index.html
