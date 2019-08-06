# Building the binary .tgz for magicleap gstreamer

`mach` downloads prebuilt gstreamer libaries, which are built using this script.

# Requirements
- Magic Leap SDK >= 0.22.0 for MacOSX
  * Download from https://creator.magicleap.com/downloads/lumin-sdk/overview
  * Install both `Lumin SDK` and `Lumin Runtime SDK` packages
- An application certificate
  * Create one on https://creator.magicleap.com in `publish` section

# Setup MacOSX
- Install python3 and HomeBrew
- pip3 install git+https://github.com/mesonbuild/meson.git
  * Requires Meson >=0.52.0, currently only in git master.
- brew install coreutils glib bison
- export PATH=/usr/local/opt/gettext/bin:/usr/local/opt/bison/bin:$PATH

# Build Instructions
- export MAGICLEAP_SDK=/path/to/mlsdk
- export MLCERT=/path/to/application.cert
- ./gstreamer.sh
