# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

# Listed all packages for different platforms in one file

CENTOSFEDORA = [
    "curl",
    "freeglut-devel",
    "libtool",
    "gcc-c++",
    "libXi-devel",
    "freetype-devel",
    "mesa-libGL-devel",
    "mesa-libEGL-devel",
    "glib2-devel",
    "libX11-devel",
    "libXrandr-devel",
    "gperf",
    "fontconfig-devel",
    "cabextract",
    "ttmkfdir",
    "python",
    "python2-virtualenv",
    "python-pip",
    "expat-devel",
    "rpm-build",
    "openssl-devel",
    "cmake",
    "bzip2-devel",
    "libXcursor-devel",
    "libXmu-devel",
    "mesa-libOSMesa-devel",
    "dbus-devel",
]

DEBIAN = [
    "git",
    "curl",
    "freeglut3-dev",
    "autoconf",
    "libfreetype6-dev",
    "libgl1-mesa-dri",
    "libglib2.0-dev",
    "xorg-dev",
    "gperf",
    "g++",
    "build-essential",
    "cmake",
    "python-pip",
    "libssl-dev",
    "libbz2-dev",
    "libosmesa6-dev",
    "libxmu6",
    "libxmu-dev",
    "libglu1-mesa-dev",
    "libgles2-mesa-dev",
    "libegl1-mesa-dev",
    "libdbus-1-dev",
]

WINDOWS_GNU = [
    "mingw-w64-x86_64-toolchain",
    "mingw-w64-x86_64-freetype",
    "mingw-w64-x86_64-icu",
    "mingw-w64-x86_64-nspr",
    "mingw-w64-x86_64-ca-certificates",
    "mingw-w64-x86_64-expat",
    "mingw-w64-x86_64-cmake",
    "tar",
    "diffutils",
    "patch",
    "patchutils",
    "make",
    "python2-setuptools",
]

WINDOWS_MSVC = [
    "cmake-3.6.1",
    "ninja-1.7.1",
    "openssl-1.0.1t-vs2015",
    "moztools-0.0.1-5",
]
