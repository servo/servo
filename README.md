# The Servo Parallel Browser Engine Project

[![Linux Build Status](https://img.shields.io/travis/servo/servo/master.svg?label=Linux%20build)](https://travis-ci.com/servo/servo)  [![Changelog #228](https://img.shields.io/badge/changelog-%23228-9E978E.svg)](https://changelog.com/podcast/228)

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

Xcode version 10.2 or above is recommended.

##### On macOS (Homebrew)

NOTE: run these steps after you've cloned the project locally.

``` sh
cd servo 
brew bundle install --file=etc/taskcluster/macos/Brewfile
brew bundle install --file=etc/taskcluster/macos/Brewfile-build
pip install virtualenv
```

#### On Debian-based Linuxes

``` sh
sudo apt install python-virtualenv python-pip
./mach bootstrap
```

If `./mach bootstrap` doesn't work, file a bug, and, run the commands below:

``` sh
sudo apt install git curl autoconf libx11-dev libfreetype6-dev libgl1-mesa-dri \
    libglib2.0-dev xorg-dev gperf g++ build-essential cmake libssl-dev \
    liblzma-dev libxmu6 libxmu-dev \
    libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
    libgles2-mesa-dev libegl1-mesa-dev libdbus-1-dev libharfbuzz-dev ccache \
    clang libunwind-dev libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev \
    libgstreamer-plugins-bad1.0-dev autoconf2.13 llvm-dev
```

Additionally, you'll need a local copy of GStreamer with a version later than 16.2. You can place it in `support/linux/gstreamer/gst`, or run `./mach bootstrap-gstreamer` to set it up. On **Ubuntu 20.04LTS**, you can use the system GStreamer if you install the necessary packages:
``` sh
sudo apt install gstreamer1.0-nice gstreamer1.0-plugins-bad
```

If you are using **Ubuntu 16.04** or **Linux Mint 18.&#42;** run `export HARFBUZZ_SYS_NO_PKG_CONFIG=1` before building to avoid an error with harfbuzz.

If you get an undefined symbol error on `gst_player_get_config` try removing `gir1.2-gst-plugins-bad-1.0` and all old versions of clang, see [#22016](https://github.com/servo/servo/issues/22016)

#### On Fedora

``` sh
sudo dnf install python3 python3-virtualenv python3-pip python3-devel
python3 ./mach bootstrap
```

If `python3 ./mach bootstrap` doesn't work, file a bug, and, run the commands below:

``` sh
sudo dnf install curl libtool gcc-c++ libXi-devel libunwind-devel \
    freetype-devel mesa-libGL-devel mesa-libEGL-devel glib2-devel libX11-devel \
    libXrandr-devel gperf fontconfig-devel cabextract ttmkfdir  expat-devel \
    rpm-build openssl-devel cmake libX11-devel libXcursor-devel \
    libXmu-devel dbus-devel ncurses-devel harfbuzz-devel \
    ccache clang clang-libs python3-devel gstreamer1-devel \
    gstreamer1-plugins-base-devel gstreamer1-plugins-bad-free-devel autoconf213
```

#### On CentOS

``` sh
sudo yum install python-virtualenv python-pip
./mach bootstrap
```

If `./mach bootstrap` doesn't work, file a bug, and, run the commands below:

``` sh
sudo yum install curl libtool gcc-c++ libXi-devel freetype-devel \
    mesa-libGL-devel mesa-libEGL-devel glib2-devel libX11-devel libXrandr-devel \
    gperf fontconfig-devel cabextract ttmkfdir python expat-devel rpm-build \
    openssl-devel cmake3 libXcursor-devel libXmu-devel \
    dbus-devel ncurses-devel python34 harfbuzz-devel \
    ccache clang clang-libs llvm-toolset-7
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
sudo zypper install libX11-devel libexpat-devel Mesa-libEGL-devel Mesa-libGL-devel cabextract cmake \
    dbus-1-devel fontconfig-devel freetype-devel gcc-c++ git glib2-devel gperf \
    harfbuzz-devel libXcursor-devel libXi-devel libXmu-devel libXrandr-devel libopenssl-devel \
    python-pip python-virtualenv rpm-build ccache llvm-clang libclang autoconf213 gstreamer-devel \
    gstreamer-plugins-base-devel gstreamer-plugins-bad-devel
```

#### On Arch Linux

``` sh
sudo pacman -S --needed base-devel git python2 python2-virtualenv python2-pip mesa cmake libxmu \
    pkg-config ttf-fira-sans harfbuzz ccache llvm clang autoconf2.13 gstreamer gstreamer-vaapi
```

#### On Gentoo Linux

```sh
sudo emerge net-misc/curl \
    media-libs/freetype media-libs/mesa dev-util/gperf \
    dev-python/virtualenv dev-python/pip dev-libs/openssl \
    media-libs/harfbuzz dev-util/ccache sys-libs/libunwind \
    x11-libs/libXmu x11-base/xorg-server sys-devel/clang \
    media-libs/gstreamer media-libs/gst-plugins-bad media-libs/gst-plugins-base
```

With the following environment variable set:
```sh
export LIBCLANG_PATH=$(llvm-config --prefix)/lib64
```

#### On Windows (MSVC)

1. Install Python 2.7 for Windows (https://www.python.org/downloads/release/python-2716/). The Windows x86-64 MSI installer is fine. This is required for the build system execution and many dependencies.

You should change the installation to install the "Add python.exe to Path" feature.

You will also need to set the `PYTHON2` environment variable, e.g., to 'C:\Python27\python.exe' by doing:
```
setx PYTHON2 "C:\Python27\python.exe" /m
```

2. Install Python 3.7 for Windows (https://www.python.org/downloads/release/python-374/). The Windows x86-64 MSI installer is fine. This is required in order to build the JavaScript engine, SpiderMonkey.

You will also need to set the `PYTHON3` environment variable, e.g., to 'C:\Python37\python.exe' by doing:
```
setx PYTHON3 "C:\Python37\python.exe" /m
```
The `/m` will set it system-wide for all future command windows.

3. Install virtualenv.

 In a normal Windows Shell (cmd.exe or "Command Prompt" from the start menu), do:
 ```
pip install virtualenv
```
 If this does not work, you may need to reboot for the changed PATH settings (by the python installer) to take effect.

4. Install the most recent [GStreamer](https://gstreamer.freedesktop.org/data/pkg/windows/) MSVC packages. You need to download the two `.msi` files for your platform from the [GStreamer](https://gstreamer.freedesktop.org/data/pkg/windows/) website and install them. The currently recommended version is 1.16.0. i.e.:

- [gstreamer-1.0-msvc-x86_64-1.16.0.msi](https://gstreamer.freedesktop.org/data/pkg/windows/1.16.0/gstreamer-1.0-msvc-x86_64-1.16.0.msi)
- [gstreamer-1.0-devel-msvc-x86_64-1.16.0.msi](https://gstreamer.freedesktop.org/data/pkg/windows/1.16.0/gstreamer-1.0-devel-msvc-x86_64-1.16.0.msi)

Note that the MinGW binaries will not work, so make sure that you install the MSVC the ones.

Note that you should ensure that _all_ components are installed from gstreamer, as we require many of the optional libraries that are not installed by default.

5. Install Git for Windows (https://git-scm.com/download/win). DO allow it to add git.exe to the PATH (default
settings for the installer are fine).

6. Install Visual Studio Community 2017 (https://www.visualstudio.com/vs/community/). You MUST add "Visual C++" to the
list of installed components as well as the "Windows Universal C runtime." They are not on by default. Visual Studio 2017 MUST installed to the default location or mach.bat will not find it.

Note that version is hard to download online and is easier to install via [Chocolatey](https://chocolatey.org/install#installing-chocolatey) with:
```
choco install -y visualstudio2017community --package-parameters="'--add Microsoft.VisualStudio.Component.Git'"
Update-SessionEnvironment #refreshing env due to Git install

#--- UWP Workload and installing Windows Template Studio ---
choco install -y visualstudio2017-workload-nativedesktop
```

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

> If you got the error `Cannot run mach in a path on a case-sensitive file system on Windows`:
> 1. Open Command Prompt or PowerShell as administrator.
> 2. Disable case-sensitive for servo path, `fsutil.exe file SetCaseSensitiveInfo X:\path\to\servo disable`

> If you got the error `DLL file `api-ms-win-crt-runtime-l1-1-0.dll` not found!` then set
> the `WindowsSdkDir` environment variable to an appropriate `Windows Kit` directory containing
> `Redist\ucrt\DLLs\x64\api-ms-win-crt-runtime-l1-1-0.dll`, for example
> `C:\Program Files (x86)\Windows Kits\10`.

> If you get the error `thread 'main' panicked at 'Unable to find libclang: "couldn\'t find any valid shared libraries matching: [\'clang.dll\', \'libclang.dll\'], set the `LIBCLANG_PATH` environment variable to a path where one of these files can be found (invalid: ... invalid DLL (64-bit))])"'`
> then `rustup` may have installed the 32-bit default target rather than the 64-bit one.
> You can find the configuration with `rustup show`, and set the default with `rustup set default-host x86_64-pc-windows-msvc`.

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

