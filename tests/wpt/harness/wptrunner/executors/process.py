# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

from .base import TestExecutor


class ProcessTestExecutor(TestExecutor):
    def __init__(self, *args, **kwargs):
        TestExecutor.__init__(self, *args, **kwargs)
        self.binary = self.browser.binary
        self.interactive = (False if self.debug_info is None
                            else self.debug_info.interactive)

    def setup(self, runner):
        self.runner = runner
        self.runner.send_message("init_succeeded")
        return True

    def is_alive(self):
        return True

    def do_test(self, test):
        raise NotImplementedError
