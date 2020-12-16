# -*- coding: utf-8 -*-
"""Skipping an entire subclass with unittest.skip() should *not* call setUpClass from a base class."""
import unittest


class Base(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        assert 0


@unittest.skip("skip all tests")
class Test(Base):
    def test_foo(self):
        assert 0
