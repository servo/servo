# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from __future__ import absolute_import, print_function

from distutils.spawn import find_executable
from distutils.version import LooseVersion
import json
import os
import platform
import shutil
import subprocess
from subprocess import PIPE

import servo.packages as packages
from servo.util import extract, download_file, host_triple


def run_as_root(command):
    if os.geteuid() != 0:
        command.insert(0, 'sudo')
    return subprocess.call(command)


def install_salt_dependencies(context, force):
    install = False
    if context.distro == 'Ubuntu':
        pkgs = ['build-essential', 'libssl-dev', 'libffi-dev', 'python-dev']
        command = ['apt-get', 'install']
        if subprocess.call(['dpkg', '-s'] + pkgs, stdout=PIPE, stderr=PIPE) != 0:
            install = True
    elif context.distro in ['CentOS', 'CentOS Linux', 'Fedora']:
        installed_pkgs = str(subprocess.check_output(['rpm', '-qa'])).replace('\n', '|')
        pkgs = ['gcc', 'libffi-devel', 'python-devel', 'openssl-devel']
        for p in pkgs:
            command = ['dnf', 'install']
            if "|{}".format(p) not in installed_pkgs:
                install = True
                break

    if install:
        if force:
            command.append('-y')
        print("Installing missing Salt dependencies...")
        run_as_root(command + pkgs)


def salt(context, force=False):
    # Ensure Salt dependencies are installed
    install_salt_dependencies(context, force)
    # Ensure Salt is installed in the virtualenv
    # It's not instaled globally because it's a large, non-required dependency,
    # and the installation fails on Windows
    print("Checking Salt installation...", end='')
    reqs_path = os.path.join(context.topdir, 'python', 'requirements-salt.txt')
    process = subprocess.Popen(
        ["pip", "install", "-q", "-I", "-r", reqs_path],
        stdout=PIPE,
        stderr=PIPE
    )
    process.wait()
    if process.returncode:
        out, err = process.communicate()
        print('failed to install Salt via pip:')
        print('Output: {}\nError: {}'.format(out, err))
        return 1
    print("done")

    salt_root = os.path.join(context.sharedir, 'salt')
    config_dir = os.path.join(salt_root, 'etc', 'salt')
    pillar_dir = os.path.join(config_dir, 'pillars')

    # In order to allow `mach bootstrap` to work from any CWD,
    # the `root_dir` must be an absolute path.
    # We place it under `context.sharedir` because
    # Salt caches data (e.g. gitfs files) in its `var` subdirectory.
    # Hence, dynamically generate the config with an appropriate `root_dir`
    # and serialize it as JSON (which is valid YAML).
    config = {
        'hash_type': 'sha384',
        'master': 'localhost',
        'root_dir': salt_root,
        'state_output': 'changes',
        'state_tabular': True,
    }
    if 'SERVO_SALTFS_ROOT' in os.environ:
        config.update({
            'fileserver_backend': ['roots'],
            'file_roots': {
                'base': [os.path.abspath(os.environ['SERVO_SALTFS_ROOT'])],
            },
        })
    else:
        config.update({
            'fileserver_backend': ['git'],
            'gitfs_env_whitelist': 'base',
            'gitfs_provider': 'gitpython',
            'gitfs_remotes': [
                'https://github.com/servo/saltfs.git',
            ],
        })

    if not os.path.exists(config_dir):
        os.makedirs(config_dir, mode=0o700)
    with open(os.path.join(config_dir, 'minion'), 'w') as config_file:
        config_file.write(json.dumps(config) + '\n')

    # Similarly, the pillar data is created dynamically
    # and temporarily serialized to disk.
    # This dynamism is not yet used, but will be in the future
    # to enable Android bootstrapping by using
    # context.sharedir as a location for Android packages.
    pillar = {
        'top.sls': {
            'base': {
                '*': ['bootstrap'],
            },
        },
        'bootstrap.sls': {
            'fully_managed': False,
        },
    }
    if os.path.exists(pillar_dir):
        shutil.rmtree(pillar_dir)
    os.makedirs(pillar_dir, mode=0o700)
    for filename in pillar:
        with open(os.path.join(pillar_dir, filename), 'w') as pillar_file:
            pillar_file.write(json.dumps(pillar[filename]) + '\n')

    cmd = [
        # sudo escapes from the venv, need to use full path
        find_executable('salt-call'),
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
        retcode = run_as_root(cmd + ['test=True'])
        if retcode != 0:
            print('Something went wrong while bootstrapping')
            return retcode

        proceed = raw_input(
            'Proposed changes are above, proceed with bootstrap? [y/N]: '
        )
        if proceed.lower() not in ['y', 'yes']:
            return 0

        print('')

    print('Running Salt bootstrap')
    retcode = run_as_root(cmd + ['--retcode-passthrough'])
    if retcode == 0:
        print('Salt bootstrapping complete')
    else:
        print('Salt bootstrapping encountered errors')
    return retcode


def windows_msvc(context, force=False):
    '''Bootstrapper for MSVC building on Windows.'''

    deps_dir = os.path.join(context.sharedir, "msvc-dependencies")
    deps_url = "https://servo-deps.s3.amazonaws.com/msvc-deps/"

    def version(package):
        return packages.WINDOWS_MSVC[package]

    def package_dir(package):
        return os.path.join(deps_dir, package, version(package))

    def check_cmake(version):
        cmake_path = find_executable("cmake")
        if cmake_path:
            cmake = subprocess.Popen([cmake_path, "--version"], stdout=PIPE)
            cmake_version = cmake.stdout.read().splitlines()[0].replace("cmake version ", "")
            if LooseVersion(cmake_version) >= LooseVersion(version):
                return True
        return False

    to_install = {}
    for package in packages.WINDOWS_MSVC:
        # Don't install CMake if it already exists in PATH
        if package == "cmake" and check_cmake(version("cmake")):
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

    if "windows-msvc" in host_triple():
        bootstrapper = windows_msvc
    elif "linux-gnu" in host_triple():
        distro, version, _ = platform.linux_distribution()
        if distro.lower() in [
            'centos',
            'centos linux',
            'debian',
            'fedora',
            'ubuntu',
        ]:
            context.distro = distro
            bootstrapper = salt

    if bootstrapper is None:
        print('Bootstrap support is not yet available for your OS.')
        return 1

    return bootstrapper(context, force=force)
