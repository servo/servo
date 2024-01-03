# The Servo Parallel Browser Engine Project

Servo is a prototype web browser engine written in the
[Rust](https://github.com/rust-lang/rust) language. It is currently developed on
64-bit macOS, 64-bit Linux, 64-bit Windows, and Android.

Servo welcomes contribution from everyone.  See
[`CONTRIBUTING.md`](CONTRIBUTING.md) and [`HACKING_QUICKSTART.md`](docs/HACKING_QUICKSTART.md)
for help getting started.

Visit the [Servo Project page](https://servo.org/) for news and guides.

## Build Setup

* [macOS](#macos)
* [Linux](#Linux)
* [Windows](#windows)
* [Android](https://github.com/servo/servo/wiki/Building-for-Android)

If these instructions fail or you would like to install dependencies
manually, try the [manual build setup][manual-build].

### macOS

- Install [Xcode](https://developer.apple.com/xcode/)
- Install [Homebrew](https://brew.sh/)
- Run `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- Run `./mach bootstrap`<br/>
  *Note: This will install the recommended version of GStreamer globally on your system.*

### Linux

- Run `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- Install Python
    - **Debian-like:** Run `sudo apt install python3-pip`
    - **Fedora:** Run `sudo dnf install python3 python3-pip python3-devel`
    - **Arch:** Run `sudo pacman -S --needed python python-pip`
    - **Gentoo:** Run `sudo emerge dev-python/pip`
- Run `./mach bootstrap`

### Windows

 - Download and run [`rustup-init.exe`](https://win.rustup.rs/) then follow the onscreen instructions.
 - Install [chocolatey](https://chocolatey.org/)
 - Install [Python 3.11](https://apps.microsoft.com/detail/9NRWMJP3717K?hl=en-US&gl=US)
 - Run `mach bootstrap`
  - *This will install CMake, Git, Ninja, and the Visual Studio 2019 Build Tools
     via choco in an Administrator console. It can take quite a while.*
  - *If you already have Visual Studio 2019 installed, this may not install all necessary components.
     Please follow the Visual Studio 2019 installation instructions in the [manual setup][manual-build].*
- Run `refreshenv`

See also [Windows Troubleshooting Tips][windows-tips].

### Cloning the Repo
Your CARGO_HOME needs to point to (or be in) the same drive as your Servo repository (See [#28530](https://github.com/servo/servo/issues/28530)).
``` sh
git clone https://github.com/servo/servo
cd servo
```

## Building

Servo is built with [Cargo](https://crates.io/), the Rust package manager.
We also use Mozilla's Mach tools to orchestrate the build and other tasks.
You can call Mach like this:

On Unix systems:
```
./mach [command] [arguments]
```
On Windows Commandline:
```
mach.bat [command] [arguments]
```
The examples below will use Unix, but the same applies to Windows.

### The Rust compiler

Servo's build system uses rustup.rs to automatically download a Rust compiler.
This is a specific version of Rust Nightly determined by the
[`rust-toolchain.toml`](https://github.com/servo/servo/blob/main/rust-toolchain.toml) file.

### Normal build

To build Servo in development mode.
This is useful for development, but the resulting binary is very slow:

``` sh
./mach build --dev
./mach run tests/html/about-mozilla.html
```

### Release build
For benchmarking, performance testing, or real-world use.
Add the `--release` flag to create an optimized build:

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

### Runtime dependencies

#### Linux

* `GStreamer` >=1.16
* `gst-plugins-bad` >=1.16
* `libXcursor`
* `libXrandr`
* `libXi`
* `libxkbcommon`
* `vulkan-loader`

## Developing

There are lots of mach commands you can use. You can list them with `./mach
--help`.


The generated documentation can be found on https://doc.servo.org/servo/index.html

[manual-build]: https://github.com/servo/servo/wiki/Building#manual-build-setup
[windows-tips]: https://github.com/servo/servo/wiki/Building#troubleshooting-the-windows-build
