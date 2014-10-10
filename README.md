The Servo Parallel Browser Project

Servo is a prototype web browser engine written in the
[Rust](https://github.com/mozilla/rust) language. It is currently developed on
64bit OS X, 64bit Linux, and Android.

Servo welcomes contribution from everyone.  See
[`CONTRIBUTING.md`](CONTRIBUTING.md) for help getting started.

## Prerequisites

Note, on systems without glfw3 packages, you can compile from source. An
example can be found in [the TravisCI install
script](etc/ci/travis.install.sh).

On OS X (homebrew):

``` sh
brew install automake pkg-config python glfw3
pip install virtualenv
```

On OS X (MacPorts):

``` sh
sudo port install python27 py27-virtualenv
```

On Debian-based Linuxes:

``` sh
sudo apt-get install curl freeglut3-dev \
    libfreetype6-dev libgl1-mesa-dri libglib2.0-dev xorg-dev \
    msttcorefonts gperf g++ cmake python-virtualenv \
    libssl-dev libglfw3-dev
```

On Fedora:

``` sh
sudo yum install curl freeglut-devel libtool gcc-c++ libXi-devel \
    freetype-devel mesa-libGL-devel glib2-devel libX11-devel libXrandr-devel gperf \
    fontconfig-devel cabextract ttmkfdir python python-virtualenv expat-devel \
    rpm-build openssl-devel glfw-devel
pushd .
cd /tmp
wget http://corefonts.sourceforge.net/msttcorefonts-2.5-1.spec
rpmbuild -bb msttcorefonts-2.5-1.spec
sudo yum install $HOME/rpmbuild/RPMS/noarch/msttcorefonts-2.5-1.noarch.rpm
popd
```

On Arch Linux:

``` sh
sudo pacman -S base-devel git python2 python2-virtualenv mesa glfw ttf-font
```

Cross-compilation for Android:

Pre-installed Android tools are needed. See wiki for
[details](https://github.com/mozilla/servo/wiki/Building-for-Android)

## The Rust compiler

Servo uses a snapshot Rust compiler to build itself. This is normally a
specific revision of Rust upstream, but sometimes has a backported patch or
two. If you'd like to know the snapshot revision of Rust which we use, see
`./rust-snapshot-hash`.

## Building

Servo is built with Cargo, the Rust package manager. We also use Mozilla's
Mach tools to orchestrate the build and other tasks.

### Normal build

``` sh
git clone https://github.com/servo/servo
cd servo
./mach build
./mach run tests/html/about-mozilla.html
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

There are lots of mach commands you can use. You can list them with `./mach
--help`.
