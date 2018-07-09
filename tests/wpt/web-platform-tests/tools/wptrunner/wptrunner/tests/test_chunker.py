import unittest
import sys
from os.path import join, dirname
from mozlog import structured

sys.path.insert(0, join(dirname(__file__), "..", "..", ".."))

from wptrunner.testloader import EqualTimeChunker
from manifest.sourcefile import SourceFile

structured.set_default_logger(structured.structuredlog.StructuredLogger("TestChunker"))


testharness_test = """<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>"""


class MockTest(object):
    default_timeout = 10

    def __init__(self, id, path, timeout=10, contents=testharness_test):
        self.id = id
        self.url = "/" + path
        self.item_type = "testharness"
        self.timeout = timeout
        self.source_file = SourceFile("/", path, "/", contents=contents)


def make_mock_manifest(*items):
    rv = []
    for test_type, dir_path, num_tests in items:
        for i in range(num_tests):
            filename = "/%i.html" % i
            rv.append((test_type,
                       dir_path + filename,
                       set([MockTest("%i.html" % i, dir_path + filename)])))
    return rv


class TestEqualTimeChunker(unittest.TestCase):

    def test_include_all(self):
        tests = make_mock_manifest(("test", "a", 10), ("test", "a/b", 10),
                                   ("test", "c", 10))

        chunk_1 = list(EqualTimeChunker(3, 1)(tests))
        chunk_2 = list(EqualTimeChunker(3, 2)(tests))
        chunk_3 = list(EqualTimeChunker(3, 3)(tests))

        self.assertEquals(tests[:10], chunk_1)
        self.assertEquals(tests[10:20], chunk_2)
        self.assertEquals(tests[20:], chunk_3)

    def test_include_all_1(self):
        tests = make_mock_manifest(("test", "a", 5), ("test", "a/b", 5),
                                   ("test", "c", 10), ("test", "d", 10))

        chunk_1 = list(EqualTimeChunker(3, 1)(tests))
        chunk_2 = list(EqualTimeChunker(3, 2)(tests))
        chunk_3 = list(EqualTimeChunker(3, 3)(tests))

        self.assertEquals(tests[:10], chunk_1)
        self.assertEquals(tests[10:20], chunk_2)
        self.assertEquals(tests[20:], chunk_3)

    def test_long(self):
        tests = make_mock_manifest(("test", "a", 100), ("test", "a/b", 1),
                                   ("test", "c", 1))

        chunk_1 = list(EqualTimeChunker(3, 1)(tests))
        chunk_2 = list(EqualTimeChunker(3, 2)(tests))
        chunk_3 = list(EqualTimeChunker(3, 3)(tests))

        self.assertEquals(tests[:100], chunk_1)
        self.assertEquals(tests[100:101], chunk_2)
        self.assertEquals(tests[101:102], chunk_3)

    def test_long_1(self):
        tests = make_mock_manifest(("test", "a", 1), ("test", "a/b", 100),
                                   ("test", "c", 1))

        chunk_1 = list(EqualTimeChunker(3, 1)(tests))
        chunk_2 = list(EqualTimeChunker(3, 2)(tests))
        chunk_3 = list(EqualTimeChunker(3, 3)(tests))

        self.assertEquals(tests[:1], chunk_1)
        self.assertEquals(tests[1:101], chunk_2)
        self.assertEquals(tests[101:102], chunk_3)

    def test_too_few_dirs(self):
        with self.assertRaises(ValueError):
            tests = make_mock_manifest(("test", "a", 1), ("test", "a/b", 100),
                                       ("test", "c", 1))
            list(EqualTimeChunker(4, 1)(tests))


if __name__ == "__main__":
    unittest.main()
