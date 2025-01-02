# mypy: allow-untyped-defs
import unittest


class Test(unittest.TestCase):
    async def test_foo(self):
        assert False
