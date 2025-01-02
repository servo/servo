import contextlib
import threading
import time
import unittest

from ..utils import MS


class ThreadTestCase(unittest.TestCase):
    @contextlib.contextmanager
    def run_in_thread(self, target):
        """
        Run ``target`` function without arguments in a thread.

        In order to facilitate writing tests, this helper lets the thread run
        for 1ms on entry and joins the thread with a 1ms timeout on exit.

        """
        thread = threading.Thread(target=target)
        thread.start()
        time.sleep(MS)
        try:
            yield
        finally:
            thread.join(MS)
            self.assertFalse(thread.is_alive())
