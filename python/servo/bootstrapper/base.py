# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

from __future__ import print_function, unicode_literals

import distutils
import subprocess


class BaseBootstrapper(object):
    """Base class for system bootstrappers."""

    def __init__(self, interactive=False):
        self.package_manager_updated = False
        self.interactive = interactive

    def ensure_system_packages(self):
        '''
        Check for missing packages.
        '''
        raise NotImplementedError('%s must implement ensure_system_packages()' %
                                  __name__)

    def install_system_packages(self):
        '''
        Install packages required to build Servo.
        '''
        raise NotImplementedError('%s must implement install_system_packages()' %
                                  __name__)

    def install_mobile_android_packages(self):
        '''
        Install packages required to build Servo for Android.
        '''
        raise NotImplementedError('Cannot bootstrap Servo for Android: '
                                  '%s does not yet implement install_mobile_android_packages()'
                                  % __name__)

    def which(self, name):
        """Python implementation of which.

        It returns the path of an executable or None if it couldn't be found.
        """
        return distutils.spawn.find_executable(name)

    def check_output(self, *args, **kwargs):
        """Run subprocess.check_output."""
        return subprocess.check_output(*args, **kwargs)

    def _ensure_package_manager_updated(self):
        if self.package_manager_updated:
            return

        self._update_package_manager()
        self.package_manager_updated = True

    def _update_package_manager(self):
        """Updates the package manager's manifests/package list.

        This should be defined in child classes.
        """
