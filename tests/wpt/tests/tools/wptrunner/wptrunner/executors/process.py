# mypy: allow-untyped-defs

from .base import TestExecutor


class ProcessTestExecutor(TestExecutor):
    def __init__(self, *args, **kwargs):
        TestExecutor.__init__(self, *args, **kwargs)
        self.binary = self.browser.binary
        self.interactive = (False if self.debug_info is None
                            else self.debug_info.interactive)

    def setup(self, runner, protocol=None):
        self.runner = runner
        self.runner.send_message("init_succeeded")
        return True

    def is_alive(self):
        return True

    def do_test(self, test):
        raise NotImplementedError
