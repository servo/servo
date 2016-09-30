# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

from __future__ import print_function

import sys

from windows_gnu import WindowsGnuBootstrapper
from windows_msvc import WindowsMsvcBootstrapper


class Bootstrapper(object):
    """Main class that performs system bootstrap."""

    def __init__(self, context):
        self.instance = None
        cls = None
        args = {}

        if sys.platform.startswith('msys'):
            cls = WindowsGnuBootstrapper

        elif sys.platform.startswith('win32'):
            cls = WindowsMsvcBootstrapper

        if cls is None:
            sys.exit('Bootstrap support is not yet available for your OS.')

        self.instance = cls(**args)
        self.instance.context = context

    def bootstrap(self, android=False, interactive=False, force=False):
        self.instance.interactive = interactive
        self.instance.force = force

        if android:
            self.instance.install_mobile_android_packages()
        elif force:
            self.instance.install_system_packages()
        else:
            self.instance.ensure_system_packages()
