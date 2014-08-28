# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from __future__ import unicode_literals

from mach.main import (
    COMMAND_ERROR,
    MODULE_ERROR
)
from mach.test.common import TestBase

from mozunit import main


class TestErrorOutput(TestBase):

    def _run_mach(self, args):
        return TestBase._run_mach(self, args, 'throw.py')

    def test_command_error(self):
        result, stdout, stderr = self._run_mach(['throw', '--message',
            'Command Error'])

        self.assertEqual(result, 1)

        self.assertIn(COMMAND_ERROR, stdout)

    def test_invoked_error(self):
        result, stdout, stderr = self._run_mach(['throw_deep', '--message',
            'Deep stack'])

        self.assertEqual(result, 1)

        self.assertIn(MODULE_ERROR, stdout)


if __name__ == '__main__':
    main()
