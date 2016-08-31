# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

from __future__ import print_function, unicode_literals

import os
import subprocess

from servo.command_base import CommandBase


class BaseBootstrapper(CommandBase):
    """Base class for system bootstrappers."""

    def __init__(self, interactive=False):
        self.package_manager_updated = False
        self.interactive = interactive

    def install_packages(self):
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
        for path in os.environ['PATH'].split(os.pathsep):
            test = os.path.join(path, name)
            if os.path.exists(test) and os.access(test, os.X_OK):
                return test

        return None

    def check_output(self, *args, **kwargs):
        """Run subprocess.check_output even if Python doesn't provide it."""
        fn = getattr(subprocess, 'check_output', BaseBootstrapper._check_output)

        return fn(*args, **kwargs)

    @staticmethod
    def _check_output(*args, **kwargs):
        """Python 2.6 compatible implementation of subprocess.check_output."""
        proc = subprocess.Popen(stdout=subprocess.PIPE, *args, **kwargs)
        output, unused_err = proc.communicate()
        retcode = proc.poll()
        if retcode:
            cmd = kwargs.get('args', args[0])
            e = subprocess.CalledProcessError(retcode, cmd)
            e.output = output
            raise e

        return output

    def _ensure_package_manager_updated(self):
        if self.package_manager_updated:
            return

        self._update_package_manager()
        self.package_manager_updated = True

    def _update_package_manager(self):
        """Updates the package manager's manifests/package list.

        This should be defined in child classes.
        """
