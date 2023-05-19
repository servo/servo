# Copyright 2023 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import os
import shutil
import subprocess
import urllib
import zipfile

from distutils.version import LooseVersion

import six
from .base import Base
from ..util import extract, download_file

DEPS_URL = "https://github.com/servo/servo-build-deps/releases/download/msvc-deps/"
DEPENDENCIES = {
    "cmake": "3.14.3",
    "llvm": "15.0.5",
    "moztools": "3.2",
    "ninja": "1.7.1",
    "nuget": "08-08-2019",
    "openssl": "111.3.0+1.1.1c-vs2017-2019-09-18",
    "gstreamer-uwp": "1.16.0.5",
    "openxr-loader-uwp": "1.0",
}


class Windows(Base):
    @staticmethod
    def cmake_already_installed(required_version: str) -> bool:
        cmake_path = shutil.which("cmake")
        if not cmake_path:
            return False

        output = subprocess.check_output([cmake_path, "--version"])
        cmake_version_output = six.ensure_str(output).splitlines()[0]
        installed_version = cmake_version_output.replace("cmake version ", "")
        return LooseVersion(installed_version) >= LooseVersion(required_version)

    @classmethod
    def prepare_file(cls, deps_dir: str, zip_path: str, full_spec: str):
        if not os.path.isfile(zip_path):
            zip_url = "{}{}.zip".format(DEPS_URL, urllib.parse.quote(full_spec))
            download_file(full_spec, zip_url, zip_path)

        print("Extracting {}...".format(full_spec), end='')
        try:
            extract(zip_path, deps_dir)
        except zipfile.BadZipfile:
            print("\nError: %s.zip is not a valid zip file, redownload..." % full_spec)
            os.remove(zip_path)
            cls.prepare_file(deps_dir, zip_path, full_spec)
        else:
            print("done")

    def _platform_bootstrap(self, cache_dir: str, _force: bool = False) -> bool:
        deps_dir = os.path.join(cache_dir, "msvc-dependencies")

        def get_package_dir(package, version) -> str:
            return os.path.join(deps_dir, package, version)

        to_install = {}
        for (package, version) in DEPENDENCIES.items():
            # Don't install CMake if it already exists in PATH
            if package == "cmake" and self.cmake_already_installed(version):
                continue

            if not os.path.isdir(get_package_dir(package, version)):
                to_install[package] = version

        if not to_install:
            return False

        print("Installing missing MSVC dependencies...")
        for (package, version) in to_install.items():
            full_spec = '{}-{}'.format(package, version)

            package_dir = get_package_dir(package, version)
            parent_dir = os.path.dirname(package_dir)
            if not os.path.isdir(parent_dir):
                os.makedirs(parent_dir)

            self.prepare_file(deps_dir, package_dir + ".zip", full_spec)

            extracted_path = os.path.join(deps_dir, full_spec)
            os.rename(extracted_path, package_dir)

        return True
