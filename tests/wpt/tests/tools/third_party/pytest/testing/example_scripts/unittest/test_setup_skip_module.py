# mypy: allow-untyped-defs
"""setUpModule is always called, even if all tests in the module are skipped"""

import unittest


def setUpModule():
    assert 0


@unittest.skip("skip all tests")
class Base(unittest.TestCase):
    def test(self):
        assert 0
