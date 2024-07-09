# The Servo Parallel Browser Engine Project

Servo is a prototype web browser engine written in the
[Rust](https://github.com/rust-lang/rust) language. It is currently developed on
64-bit macOS, 64-bit Linux, 64-bit Windows, and Android.

Servo welcomes contribution from everyone. Check out [The Servo Book](https://book.servo.org) to get started, or go to [servo.org](https://servo.org/) for news and guides.

## Getting started

For more detailed build instructions, see the Servo book under [Setting up your environment](https://book.servo.org/hacking/setting-up-your-environment.html), [Building Servo](https://book.servo.org/hacking/building-servo.html), and [Building for Android](https://book.servo.org/hacking/building-for-android.html).

### macOS

- Download and install [`python`](https://www.python.org/downloads/macos/), [Xcode](https://developer.apple.com/xcode/), and [`brew`](https://brew.sh/)
- Install `rustup`: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- Restart your shell to make sure `cargo` is available
- Install the other dependencies: `./mach bootstrap`
- Build servoshell: `./mach build`

### Linux

- Install `curl` and `python`:
  - Arch: `sudo pacman -S --needed curl python python-pip`
  - Debian, Ubuntu: `sudo apt install curl python3-pip python3-venv`
  - Fedora: `sudo dnf install curl python3 python3-pip python3-devel`
  - Gentoo: `sudo emerge net-misc/curl dev-python/pip`
- Install `rustup`: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- Restart your shell to make sure `cargo` is available
- Install the other dependencies: `./mach bootstrap`
- Build servoshell: `./mach build`

### Windows

- Download and install [`python`](https://www.python.org/downloads/windows/), [`choco`](https://chocolatey.org/install#individual), and [`rustup`](https://win.rustup.rs/)
  - Be sure to select *Quick install via the Visual Studio Community installer*
- In the Visual Studio Installer, ensure the following components are installed:
  - **Windows 10 SDK (10.0.19041.0)** (`Microsoft.VisualStudio.Component.Windows10SDK.19041`)
  - **MSVC v143 - VS 2022 C++ x64/x86 build tools (Latest)** (`Microsoft.VisualStudio.Component.VC.Tools.x86.x64`)
  - **C++ ATL for latest v143 build tools (x86 & x64)** (`Microsoft.VisualStudio.Component.VC.ATL`)
  - **C++ MFC for latest v143 build tools (x86 & x64)** (`Microsoft.VisualStudio.Component.VC.ATLMFC`)
- Restart your shell to make sure `cargo` is available
- Install the other dependencies: `.\mach bootstrap`
- Build servoshell: `.\mach build`

### Android

- Ensure that the following environment variables are set:
  - `ANDROID_SDK_ROOT`
  - `ANDROID_NDK_ROOT`: `$ANDROID_SDK_ROOT/ndk/25.2.9519653/`
 `ANDROID_SDK_ROOT` can be any directory (such as `~/android-sdk`).
  All of the Android build dependencies will be installed there.
- Install the latest version of the [Android command-line
  tools](https://developer.android.com/studio#command-tools) to
  `$ANDROID_SDK_ROOT/cmdline-tools/latest`.
- Run the following command to install the necessary components:
  ```shell
  sudo $ANDROID_SDK_ROOT/cmdline-tools/latest/bin/sdkmanager --install
   "build-tools;33.0.2" \
   "emulator" \
   "ndk;25.2.9519653" \
   "platform-tools" \
   "platforms;android-33" \
   "system-images;android-33;google_apis;x86_64"
  ```
- Follow the instructions above for the platform you are building on
