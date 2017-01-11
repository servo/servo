# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from __future__ import absolute_import, print_function

import distutils.spawn
import json
import os
import platform
import subprocess
import sys

from servo.util import extract, download_file
import servo.packages as packages


def salt(context, force=False):
    salt_root = os.path.join(context.sharedir, 'salt')
    config_dir = os.path.join(salt_root, 'etc', 'salt')
    pillar_dir = os.path.join(context.topdir, 'etc', 'salt', 'pillars')

    config = {
        'fileserver_backend': ['git'],
        # TODO: replace aneeshusa remote with servo remote,
        # remove gitfs_base after relevant saltfs PR is merged
        'gitfs_base': 'enable-mach-bootstrap',
        'gitfs_env_whitelist': 'base',
        'gitfs_provider': 'gitpython',
        'gitfs_remotes': [
            'https://github.com/aneeshusa/servo-saltfs.git',
        ],
        'hash_type': 'sha384',
        'master': 'localhost',
        'root_dir': salt_root,
        'state_output': 'changes',
        'state_tabular': True,
    }

    if not os.path.exists(config_dir):
        os.makedirs(config_dir, mode=0o700)
    # In order to allow `mach bootstrap` to work from any CWD,
    # the `root_dir` must be an absolute path.
    # We place it under `context.sharedir` because
    # Salt caches data (e.g. gitfs files) in its `var` subdirectory.
    # Hence, dynamically generate the config with an appropriate `root_dir`
    # and serialize it as JSON (which is valid YAML).
    with open(os.path.join(config_dir, 'minion'), 'w') as config_file:
        config_file.write(json.dumps(config))

    cmd = [
        'sudo',
        # sudo escapes from the venv, need to use full path
        distutils.spawn.find_executable('salt-call'),
        '--local',
        '--config-dir={}'.format(config_dir),
        '--pillar-root={}'.format(pillar_dir),
        'state.apply',
        'servo-build-dependencies',
    ]

    if not force:
        print('Running bootstrap in dry-run mode to show changes')
        # Because `test=True` mode runs each state individually without
        # considering how required/previous states affect the system,
        # it will often report states with requisites as failing due
        # to the requisites not actually being run,
        # even though these are spurious and will succeed during
        # the actual highstate.
        # Hence `--retcode-passthrough` is not helpful in dry-run mode,
        # so only detect failures of the actual salt-call binary itself.
        retcode = subprocess.call(cmd + ['test=True'])
        if retcode != 0:
            print('Something went wrong while bootstrapping')
            return retcode

        proceed = raw_input(
            'Proposed changes are above, proceed with bootstrap? [y/N]: '
        )
        if proceed not in ['y', 'yes', 'Y', 'Yes', 'YES']:
            return 0

        print('')

    print('Running Salt bootstrap')
    retcode = subprocess.call(cmd + ['--retcode-passthrough'])
    if retcode == 0:
        print('Salt bootstrapping complete')
    else:
        print('Salt bootstrapping encountered errors')
    return retcode


def windows_gnu(context, force=False):
    '''Bootstrapper for msys2 based environments for building in Windows.'''

    if not distutils.spawn.find_executable('pacman'):
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
    subprocess.check_call(command + packages.WINDOWS_GNU)

    # Downgrade GCC to 5.4.0-1
    gcc_pkgs = ["gcc", "gcc-ada", "gcc-fortran", "gcc-libgfortran", "gcc-libs", "gcc-objc"]
    gcc_version = "5.4.0-1"
    mingw_url = "http://repo.msys2.org/mingw/x86_64/mingw-w64-x86_64-{}-{}-any.pkg.tar.xz"
    gcc_list = [mingw_url.format(gcc, gcc_version) for gcc in gcc_pkgs]

    downgrade_command = ['pacman', '--upgrade']  # Note: `--upgrade` also does downgrades
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
        if package == "cmake" and distutils.spawn.find_executable(package):
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

    if sys.platform.startswith('msys'):
        bootstrapper = windows_gnu
    elif sys.platform.startswith('win32'):
        bootstrapper = windows_msvc
    elif sys.platform.startswith('linux'):
        distro, version, _ = platform.linux_distribution()
        if distro == 'Ubuntu' and version == '14.04':
            bootstrapper = salt

    if bootstrapper is None:
        sys.exit('Bootstrap support is not yet available for your OS.')

    return bootstrapper(context, force=force)
