# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from __future__ import unicode_literals

from StringIO import StringIO
import os
import unittest

from mach.main import Mach
from mach.base import CommandContext

here = os.path.abspath(os.path.dirname(__file__))

class TestBase(unittest.TestCase):
    provider_dir = os.path.join(here, 'providers')

    def _run_mach(self, args, provider_file=None, entry_point=None, context_handler=None):
        m = Mach(os.getcwd())
        m.define_category('testing', 'Mach unittest', 'Testing for mach core', 10)
        m.populate_context_handler = context_handler

        if provider_file:
            m.load_commands_from_file(os.path.join(self.provider_dir, provider_file))

        if entry_point:
            m.load_commands_from_entry_point(entry_point)

        stdout = StringIO()
        stderr = StringIO()
        stdout.encoding = 'UTF-8'
        stderr.encoding = 'UTF-8'

        try:
            result = m.run(args, stdout=stdout, stderr=stderr)
        except SystemExit:
            result = None

        return (result, stdout.getvalue(), stderr.getvalue())
