# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

from base import BaseBootstrapper


class CentOSFedoraBootstrapper(BaseBootstrapper):
    from packages import CENTOSFEDORA as desktop_deps

    def __init__(self, distro, version, dist_id, **kwargs):
        BaseBootstrapper.__init__(self, **kwargs)

        self.distro = distro
        self.version = version
        self.dist_id = dist_id

    def ensure_system_packages(self):
        install_packages = []
        installed_list = str(self.check_output(['rpm', '-qa'])).replace('\n', '|')
        install_packages = []
        for p in self.desktop_deps:
            if not "|" + p in installed_list:
                install_packages += [p]
        if install_packages:
            print "Installing missing packages..."
            self.install_system_packages(install_packages)

    def install_system_packages(self, packages=desktop_deps):
        self.dnf_install(*packages)

    def install_mobile_android_packages(self):
        raise NotImplementedError('Bootstrap support for Android not yet available.')

    def install_virtualenv(self):
        self.dnf_install(*["python-pip", "python-virtualenv"])

    def dnf_install(self, *packages):
        if self.which('dnf'):
            command = ['dnf', 'reinstall' if self.force else 'install']
        else:
            command = ['yum', 'install']

        if not self.interactive:
            command.append('-y')
        command.extend(packages)

        self.run_as_root(command)
