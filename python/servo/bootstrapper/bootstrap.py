# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

from __future__ import print_function

import platform
import sys

from centosfedora import CentOSFedoraBootstrapper
from debian import DebianBootstrapper
from windows_gnu import WindowsGnuBootstrapper
from windows_msvc import WindowsMsvcBootstrapper


DEBIAN_DISTROS = (
    'Debian',
    'debian',
    'Ubuntu',
    # Most Linux Mint editions are based on Ubuntu. One is based on Debian.
    # The difference is reported in dist_id from platform.linux_distribution.
    # But it doesn't matter since we share a bootstrapper between Debian and
    # Ubuntu.
    'Mint',
    'LinuxMint',
    'Elementary OS',
    'Elementary',
    '"elementary OS"',
)


class Bootstrapper(object):
    """Main class that performs system bootstrap."""

    def __init__(self):
        self.instance = None
        cls = None
        args = {}

        if sys.platform.startswith('linux'):
            distro, version, dist_id = platform.linux_distribution()

            if distro in ('CentOS', 'CentOS Linux', 'Fedora'):
                cls = CentOSFedoraBootstrapper
                args['distro'] = distro
            elif distro in DEBIAN_DISTROS:
                cls = DebianBootstrapper
            else:
                sys.exit('Bootstrap support for this Linux distro not yet available.')

            args['version'] = version
            args['dist_id'] = dist_id

        elif sys.platform.startswith('msys'):
            cls = WindowsGnuBootstrapper

        elif sys.platform.startswith('win32'):
            cls = WindowsMsvcBootstrapper

        if cls is None:
            sys.exit('Bootstrap support is not yet available for your OS.')

        self.instance = cls(**args)

    def bootstrap(self, android=False, interactive=False, force=False):
        self.instance.interactive = interactive
        self.instance.force = force

        if force:
            self.instance.install_system_packages()
        else:
            self.instance.ensure_system_packages()

        if android:
            self.instance.install_mobile_android_packages()

        print
