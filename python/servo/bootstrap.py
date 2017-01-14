# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from __future__ import absolute_import, print_function

from distutils.spawn import find_executable
import os
import subprocess
import sys

import servo.packages as packages
from servo.util import extract, download_file, host_triple


def windows_gnu(context, force=False):
    '''Bootstrapper for msys2 based environments for building in Windows.'''

    if not find_executable('pacman'):
        print(
            'The Windows GNU bootstrapper only works with msys2 with pacman. '
            'Get msys2 at http://msys2.github.io/'
        )
        return 1

    # Ensure repositories are up to date
    command = ['pacman', '--sync', '--refresh']
    subprocess.check_call(command)

    # Install packages
    command = ['pacman', '--sync', '--needed']
    if force:
        command.append('--noconfirm')
    subprocess.check_call(command + list(packages.WINDOWS_GNU))

    # Downgrade GCC to 5.4.0-1
    gcc_pkgs = ["gcc", "gcc-ada", "gcc-fortran", "gcc-libgfortran", "gcc-libs", "gcc-objc"]
    gcc_version = "5.4.0-1"
    mingw_url = "http://repo.msys2.org/mingw/x86_64/mingw-w64-x86_64-{}-{}-any.pkg.tar.xz"
    gcc_list = [mingw_url.format(gcc, gcc_version) for gcc in gcc_pkgs]

    # Note: `--upgrade` also does downgrades
    downgrade_command = ['pacman', '--upgrade']
    if force:
        downgrade_command.append('--noconfirm')
    subprocess.check_call(downgrade_command + gcc_list)


def windows_msvc(context, force=False):
    '''Bootstrapper for MSVC building on Windows.'''

    deps_dir = os.path.join(context.sharedir, "msvc-dependencies")
    deps_url = "https://servo-rust.s3.amazonaws.com/msvc-deps/"

    def version(package):
        return packages.WINDOWS_MSVC[package]

    def package_dir(package):
        return os.path.join(deps_dir, package, version(package))

    to_install = {}
    for package in packages.WINDOWS_MSVC:
        # Don't install CMake if it already exists in PATH
        if package == "cmake" and find_executable(package):
            continue

        if not os.path.isdir(package_dir(package)):
            to_install[package] = version(package)

    if not to_install:
        return 0

    print("Installing missing MSVC dependencies...")
    for package in to_install:
        full_spec = '{}-{}'.format(package, version(package))

        parent_dir = os.path.dirname(package_dir(package))
        if not os.path.isdir(parent_dir):
            os.makedirs(parent_dir)

        zip_path = package_dir(package) + ".zip"
        if not os.path.isfile(zip_path):
            zip_url = "{}{}.zip".format(deps_url, full_spec)
            download_file(full_spec, zip_url, zip_path)

        print("Extracting {}...".format(full_spec), end='')
        extract(zip_path, deps_dir)
        print("done")

        extracted_path = os.path.join(deps_dir, full_spec)
        os.rename(extracted_path, package_dir(package))

    return 0


def bootstrap(context, force=False):
    '''Dispatches to the right bootstrapping function for the OS.'''

    bootstrapper = None

    if "windows-gnu" in host_triple():
        bootstrapper = windows_gnu
    elif "windows-msvc" in host_triple():
        bootstrapper = windows_msvc

    if bootstrapper is None:
        print('Bootstrap support is not yet available for your OS.')
        return 1

    return bootstrapper(context, force=force)
