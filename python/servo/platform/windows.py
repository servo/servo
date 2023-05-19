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
import tempfile
from typing import Optional
import urllib
import zipfile
from distutils.version import LooseVersion

import six
from .. import util
from .base import Base

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

URL_BASE = "https://gstreamer.freedesktop.org/data/pkg/windows/1.16.0/"
GSTREAMER_URL = f"{URL_BASE}/gstreamer-1.0-msvc-x86_64-1.16.0.msi"
GSTREAMER_DEVEL_URL = f"{URL_BASE}/gstreamer-1.0-devel-msvc-x86_64-1.16.0.msi"
DEPENDENCIES_DIR = os.path.join(util.get_target_dir(), "dependencies")


class Windows(Base):
    def __init__(self, triple: str):
        super().__init__(triple)
        self.is_windows = True

    def executable_suffix(self):
        return ".exe"

    def library_path_variable_name(self):
        return "LIB"

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
            util.download_file(full_spec, zip_url, zip_path)

        print("Extracting {}...".format(full_spec), end="")
        try:
            util.extract(zip_path, deps_dir)
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
        for package, version in DEPENDENCIES.items():
            # Don't install CMake if it already exists in PATH
            if package == "cmake" and self.cmake_already_installed(version):
                continue

            if not os.path.isdir(get_package_dir(package, version)):
                to_install[package] = version

        if not to_install:
            return False

        print("Installing missing MSVC dependencies...")
        for package, version in to_install.items():
            full_spec = "{}-{}".format(package, version)

            package_dir = get_package_dir(package, version)
            parent_dir = os.path.dirname(package_dir)
            if not os.path.isdir(parent_dir):
                os.makedirs(parent_dir)

            self.prepare_file(deps_dir, package_dir + ".zip", full_spec)

            extracted_path = os.path.join(deps_dir, full_spec)
            os.rename(extracted_path, package_dir)

        return True

    def gstreamer_root(self, cross_compilation_target: Optional[str]) -> Optional[str]:
        build_target_triple = cross_compilation_target or self.triple
        gst_arch_names = {
            "x86_64": "X86_64",
            "x86": "X86",
            "aarch64": "ARM64",
        }
        gst_arch_name = gst_arch_names[build_target_triple.split("-")[0]]

        # The bootstraped version of GStreamer always takes precedance of the installed vesion.
        prepackaged_root = os.path.join(
            DEPENDENCIES_DIR, "gstreamer", "1.0", gst_arch_name
        )
        if os.path.exists(os.path.join(prepackaged_root, "bin", "ffi-7.dll")):
            return prepackaged_root

        # The installed version of GStreamer often sets an environment variable pointing to
        # the install location.
        root_from_env = os.environ.get(f"GSTREAMER_1_0_ROOT_{gst_arch_name}")
        if root_from_env:
            return root_from_env

        # If all else fails, look for an installation in the default install directory.
        default_root = os.path.join("C:\\gstreamer\\1.0", gst_arch_name)
        if os.path.exists(os.path.join(default_root, "bin", "ffi-7.dll")):
            return default_root

        return None

    def is_gstreamer_installed(self, cross_compilation_target: Optional[str]) -> bool:
        return self.gstreamer_root(cross_compilation_target) is not None

    def _platform_bootstrap_gstreamer(self, force: bool) -> bool:
        if not force and self.is_gstreamer_installed(cross_compilation_target=None):
            return False

        if "x86_64" not in self.triple:
            print("Bootstrapping gstreamer not supported on "
                  "non-x86-64 Windows. Please install manually")
            return False

        with tempfile.TemporaryDirectory() as temp_dir:
            libs_msi = os.path.join(temp_dir, GSTREAMER_URL.rsplit("/", maxsplit=1)[-1])
            devel_msi = os.path.join(
                temp_dir, GSTREAMER_DEVEL_URL.rsplit("/", maxsplit=1)[-1]
            )

            util.download_file("GStreamer libraries", GSTREAMER_URL, libs_msi)
            util.download_file(
                "GStreamer development support", GSTREAMER_DEVEL_URL, devel_msi
            )

            print(f"Installing GStreamer packages to {DEPENDENCIES_DIR}...")
            os.makedirs(DEPENDENCIES_DIR, exist_ok=True)
            common_args = [
                f"TARGETDIR={DEPENDENCIES_DIR}",  # Install destination
                "/qn",  # Quiet mode
            ]
            subprocess.check_call(["msiexec", "/a", libs_msi] + common_args)
            subprocess.check_call(["msiexec", "/a", devel_msi] + common_args)

            assert self.is_gstreamer_installed(cross_compilation_target=None)
            return True
