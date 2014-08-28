# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.
from __future__ import unicode_literals

from mach.decorators import (
    CommandProvider,
    Command,
)

@CommandProvider
class ConditionsProvider(object):
    @Command('cmd_foo', category='testing')
    def run_foo(self):
        pass
