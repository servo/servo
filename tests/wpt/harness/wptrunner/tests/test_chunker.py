# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

import unittest
import sys
from os.path import join, dirname
from mozlog import structured

import pytest

sys.path.insert(0, join(dirname(__file__), "..", ".."))

from wptrunner.testloader import EqualTimeChunker

structured.set_default_logger(structured.structuredlog.StructuredLogger("TestChunker"))

class MockTest(object):
    def __init__(self, id, timeout=10):
        self.id = id
        self.item_type = "testharness"
        self.timeout = timeout


def make_mock_manifest(*items):
    rv = []
    for dir_path, num_tests in items:
        for i in range(num_tests):
            rv.append((dir_path + "/%i.test" % i, set([MockTest(i)])))
    return rv


class TestEqualTimeChunker(unittest.TestCase):

    def test_include_all(self):
        tests = make_mock_manifest(("a", 10), ("a/b", 10), ("c", 10))

        chunk_1 = list(EqualTimeChunker(3, 1)(tests))
        chunk_2 = list(EqualTimeChunker(3, 2)(tests))
        chunk_3 = list(EqualTimeChunker(3, 3)(tests))

        self.assertEquals(tests[:10], chunk_1)
        self.assertEquals(tests[10:20], chunk_2)
        self.assertEquals(tests[20:], chunk_3)

    def test_include_all_1(self):
        tests = make_mock_manifest(("a", 5), ("a/b", 5), ("c", 10), ("d", 10))

        chunk_1 = list(EqualTimeChunker(3, 1)(tests))
        chunk_2 = list(EqualTimeChunker(3, 2)(tests))
        chunk_3 = list(EqualTimeChunker(3, 3)(tests))

        self.assertEquals(tests[:10], chunk_1)
        self.assertEquals(tests[10:20], chunk_2)
        self.assertEquals(tests[20:], chunk_3)

    def test_long(self):
        tests = make_mock_manifest(("a", 100), ("a/b", 1), ("c", 1))

        chunk_1 = list(EqualTimeChunker(3, 1)(tests))
        chunk_2 = list(EqualTimeChunker(3, 2)(tests))
        chunk_3 = list(EqualTimeChunker(3, 3)(tests))

        self.assertEquals(tests[:100], chunk_1)
        self.assertEquals(tests[100:101], chunk_2)
        self.assertEquals(tests[101:102], chunk_3)

    def test_long_1(self):
        tests = make_mock_manifest(("a", 1), ("a/b", 100), ("c", 1))

        chunk_1 = list(EqualTimeChunker(3, 1)(tests))
        chunk_2 = list(EqualTimeChunker(3, 2)(tests))
        chunk_3 = list(EqualTimeChunker(3, 3)(tests))

        self.assertEquals(tests[:1], chunk_1)
        self.assertEquals(tests[1:101], chunk_2)
        self.assertEquals(tests[101:102], chunk_3)

    def test_too_few_dirs(self):
        with self.assertRaises(ValueError):
            tests = make_mock_manifest(("a", 1), ("a/b", 100), ("c", 1))
            list(EqualTimeChunker(4, 1)(tests))


if __name__ == "__main__":
    unittest.main()
