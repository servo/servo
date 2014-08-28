# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from __future__ import unicode_literals

import os

from mach.base import MachError
from mach.main import Mach
from mach.test.common import TestBase

from mozunit import main


def _populate_context(context, key=None):
    if key is None:
        return
    if key == 'foo':
        return True
    if key == 'bar':
        return False
    raise AttributeError(key)

class TestConditions(TestBase):
    """Tests for conditionally filtering commands."""

    def _run_mach(self, args, context_handler=None):
        return TestBase._run_mach(self, args, 'conditions.py',
                                  context_handler=context_handler)


    def test_conditions_pass(self):
        """Test that a command which passes its conditions is runnable."""

        self.assertEquals((0, '', ''), self._run_mach(['cmd_foo']))
        self.assertEquals((0, '', ''), self._run_mach(['cmd_foo_ctx'], _populate_context))

    def test_invalid_context_message(self):
        """Test that commands which do not pass all their conditions
        print the proper failure message."""

        def is_bar():
            """Bar must be true"""
        fail_conditions = [is_bar]

        for name in ('cmd_bar', 'cmd_foobar'):
            result, stdout, stderr = self._run_mach([name])
            self.assertEquals(1, result)

            fail_msg = Mach._condition_failed_message(name, fail_conditions)
            self.assertEquals(fail_msg.rstrip(), stdout.rstrip())

        for name in ('cmd_bar_ctx', 'cmd_foobar_ctx'):
            result, stdout, stderr = self._run_mach([name], _populate_context)
            self.assertEquals(1, result)

            fail_msg = Mach._condition_failed_message(name, fail_conditions)
            self.assertEquals(fail_msg.rstrip(), stdout.rstrip())

    def test_invalid_type(self):
        """Test that a condition which is not callable raises an exception."""

        m = Mach(os.getcwd())
        m.define_category('testing', 'Mach unittest', 'Testing for mach core', 10)
        self.assertRaises(MachError, m.load_commands_from_file,
                os.path.join(self.provider_dir, 'conditions_invalid.py'))

    def test_help_message(self):
        """Test that commands that are not runnable do not show up in help."""

        result, stdout, stderr = self._run_mach(['help'], _populate_context)
        self.assertIn('cmd_foo', stdout)
        self.assertNotIn('cmd_bar', stdout)
        self.assertNotIn('cmd_foobar', stdout)
        self.assertIn('cmd_foo_ctx', stdout)
        self.assertNotIn('cmd_bar_ctx', stdout)
        self.assertNotIn('cmd_foobar_ctx', stdout)


if __name__ == '__main__':
    main()
