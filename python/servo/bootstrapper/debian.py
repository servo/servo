# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import subprocess

from base import BaseBootstrapper


class DebianBootstrapper(BaseBootstrapper):
    '''Bootstrapper for Debian-based distributions.'''

    from packages import DEBIAN as desktop_deps

    def __init__(self, version, dist_id, **kwargs):
        BaseBootstrapper.__init__(self, **kwargs)
        # Find Python virtualenv package name
        venv = subprocess.Popen(['apt-cache', 'policy', 'virtualenv'], stdout=subprocess.PIPE)
        self.virtualenv = 'virtualenv' if "virtualenv:" in venv.stdout.read() else 'python-virtualenv'
        self.desktop_deps += [self.virtualenv]

        self.version = version
        self.dist_id = dist_id

    def ensure_system_packages(self):
        printed = False

        for package in self.desktop_deps:
            if self.run_check(['dpkg', '-s', package]):
                if not printed:
                    print "Updating package manager..."
                    self._ensure_package_manager_updated()
                    print "Installing missing packages..."
                    printed = True

                print "Installing %s..." % package
                self.apt_install(package)

    def install_system_packages(self, packages=desktop_deps):
        self._ensure_package_manager_updated()
        self.apt_install(*packages)

    def install_mobile_android_packages(self):
        raise NotImplementedError('Bootstrap support for Android not yet available.')

    def install_virtualenv(self):
        self.apt_install(*["python-pip", self.virtualenv])

    def _update_package_manager(self):
        self.apt_update()

    def apt_install(self, *packages):
        command = ['apt-get', 'install']
        if not self.interactive:
            command.append('-y')
        if self.force:
            command.append('--reinstall')
        command.extend(packages)
        self.run_as_root(command)

    def apt_update(self):
        command = ['apt-get', 'update']
        if not self.interactive:
            command.append('-y')
        self.run_as_root(command)
