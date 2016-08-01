# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import sys
import subprocess

from base import BaseBootstrapper
from packages import WINDOWS_GNU as deps


class WindowsGnuBootstrapper(BaseBootstrapper):
    '''Bootstrapper for msys2 based environments for building in Windows.'''

    def __init__(self, **kwargs):
        BaseBootstrapper.__init__(self, **kwargs)

        if not self.which('pacman'):
            raise NotImplementedError('The Windows bootstrapper only works with msys2 with pacman. Get msys2 at '
                                      'http://msys2.github.io/')

    def ensure_system_packages(self):
        install_packages = []
        for p in deps:
            command = ['pacman', '-Qs', p]
            if self.run_check(command):
                install_packages += [p]
        if install_packages:
            install_packages(install_packages)

    def install_system_packages(self, packages=deps):
        self._ensure_package_manager_updated()
        self.pacman_install(*packages)

    def install_mobile_android_packages(self):
        sys.exit('We do not support building Android on Windows. Sorry!')

    def _update_package_manager(self):
        self.pacman_update()

    def run(self, command):
        subprocess.check_call(command, stdin=sys.stdin)

    def run_check(self, command):
        return subprocess.call(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE)

    def pacman_update(self):
        command = ['pacman', '--sync', '--refresh']
        self.run(command)

    def pacman_upgrade(self):
        command = ['pacman', '--sync', '--refresh', '--sysupgrade']
        self.run(command)

    def pacman_install(self, *packages):
        command = ['pacman', '--sync']
        if not self.force:
            command.append('--needed')
        if not self.interactive:
            command.append('--noconfirm')
        command.extend(packages)
        self.run(command)

        # downgrade GCC to 5.4.0-1
        gcc_type = ["gcc", "gcc-ada", "gcc-fortran", "gcc-libgfortran", "gcc-libs", "gcc-objc"]
        gcc_version = "5.4.0-1"
        mingw_url = "http://repo.msys2.org/mingw/x86_64/mingw-w64-x86_64-{}-{}-any.pkg.tar.xz"
        gcc_list = []
        for gcc in gcc_type:
            gcc_list += [mingw_url.format(gcc, gcc_version)]
        downgrade_command = ['pacman', '-U']
        if not self.interactive:
            downgrade_command.append('--noconfirm')
        downgrade_command.extend(gcc_list)
        self.run(downgrade_command)
