import unittest

from websockets.sync.utils import *

from ..utils import MS


class DeadlineTests(unittest.TestCase):
    def test_timeout_pending(self):
        """timeout returns remaining time if deadline is in the future."""
        deadline = Deadline(MS)
        timeout = deadline.timeout()
        self.assertGreater(timeout, 0)
        self.assertLess(timeout, MS)

    def test_timeout_elapsed_exception(self):
        """timeout raises TimeoutError if deadline is in the past."""
        deadline = Deadline(-MS)
        with self.assertRaises(TimeoutError):
            deadline.timeout()

    def test_timeout_elapsed_no_exception(self):
        """timeout doesn't raise TimeoutError when raise_if_elapsed is disabled."""
        deadline = Deadline(-MS)
        timeout = deadline.timeout(raise_if_elapsed=False)
        self.assertGreater(timeout, -2 * MS)
        self.assertLess(timeout, -MS)

    def test_no_timeout(self):
        """timeout returns None when no deadline is set."""
        deadline = Deadline(None)
        timeout = deadline.timeout()
        self.assertIsNone(timeout, None)
