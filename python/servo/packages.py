# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

WINDOWS_GNU = set([
    "diffutils",
    "make",
    "mingw-w64-x86_64-toolchain",
    "mingw-w64-x86_64-icu",
    "mingw-w64-x86_64-nspr",
    "mingw-w64-x86_64-ca-certificates",
    "mingw-w64-x86_64-expat",
    "mingw-w64-x86_64-cmake",
    "patch",
    "patchutils",
    "python2-setuptools",
    "tar",
])

WINDOWS_MSVC = {
    "cmake": "3.6.1",
    "moztools": "0.0.1-5",
    "ninja": "1.7.1",
    "openssl": "1.0.1t-vs2015",
}
