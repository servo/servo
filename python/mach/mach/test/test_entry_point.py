# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.
from __future__ import unicode_literals

import imp
import os
import sys

from mach.base import MachError
from mach.test.common import TestBase
from mock import patch

from mozunit import main


here = os.path.abspath(os.path.dirname(__file__))

class Entry():
    """Stub replacement for pkg_resources.EntryPoint"""
    def __init__(self, providers):
        self.providers = providers

    def load(self):
        def _providers():
            return self.providers
        return _providers

class TestEntryPoints(TestBase):
    """Test integrating with setuptools entry points"""
    provider_dir = os.path.join(here, 'providers')

    def _run_mach(self):
        return TestBase._run_mach(self, ['help'], entry_point='mach.providers')

    @patch('pkg_resources.iter_entry_points')
    def test_load_entry_point_from_directory(self, mock):
        # Ensure parent module is present otherwise we'll (likely) get
        # an error due to unknown parent.
        if b'mach.commands' not in sys.modules:
            mod = imp.new_module(b'mach.commands')
            sys.modules[b'mach.commands'] = mod

        mock.return_value = [Entry(['providers'])]
        # Mach error raised due to conditions_invalid.py
        with self.assertRaises(MachError):
            self._run_mach()

    @patch('pkg_resources.iter_entry_points')
    def test_load_entry_point_from_file(self, mock):
        mock.return_value = [Entry([os.path.join('providers', 'basic.py')])]

        result, stdout, stderr = self._run_mach()
        self.assertIsNone(result)
        self.assertIn('cmd_foo', stdout)


# Not enabled in automation because tests are failing.
#if __name__ == '__main__':
#    main()
