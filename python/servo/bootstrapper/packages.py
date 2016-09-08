# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

# Listed all packages for different platforms in one file

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
