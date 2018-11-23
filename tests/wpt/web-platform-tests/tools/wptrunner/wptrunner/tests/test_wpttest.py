import os
import sys
from io import BytesIO

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", ".."))

from wptrunner import manifestexpected, wpttest
from .test_chunker import make_mock_manifest

dir_ini_0 = """\
prefs: [a:b]
"""

dir_ini_1 = """\
prefs: [@Reset, b:c]
max-asserts: 2
min-asserts: 1
tags: [b, c]
"""

dir_ini_2 = """\
lsan-max-stack-depth: 42
"""

test_0 = """\
[0.html]
  prefs: [c:d]
  max-asserts: 3
  tags: [a, @Reset]
"""

test_1 = """\
[1.html]
  prefs:
    if os == 'win': [a:b, c:d]
  expected:
    if os == 'win': FAIL
"""

test_2 = """\
[2.html]
  lsan-max-stack-depth: 42
"""


def test_metadata_inherit():
    tests = make_mock_manifest(("test", "a", 10), ("test", "a/b", 10),
                               ("test", "c", 10))

    inherit_metadata = [
        manifestexpected.static.compile(
            BytesIO(item),
            {},
            data_cls_getter=lambda x,y: manifestexpected.DirectoryManifest)
        for item in [dir_ini_0, dir_ini_1]]
    test_metadata = manifestexpected.static.compile(BytesIO(test_0),
                                                    {},
                                                    data_cls_getter=manifestexpected.data_cls_getter,
                                                    test_path="a",
                                                    url_base="")

    test = tests[0][2].pop()
    test_obj = wpttest.from_manifest(tests, test, inherit_metadata, test_metadata.get_test(test.id))
    assert test_obj.max_assertion_count == 3
    assert test_obj.min_assertion_count == 1
    assert test_obj.prefs == {"b": "c", "c": "d"}
    assert test_obj.tags == {"a", "dir:a"}


def test_conditional():
    tests = make_mock_manifest(("test", "a", 10), ("test", "a/b", 10),
                               ("test", "c", 10))

    test_metadata = manifestexpected.static.compile(BytesIO(test_1),
                                                    {"os": "win"},
                                                    data_cls_getter=manifestexpected.data_cls_getter,
                                                    test_path="a",
                                                    url_base="")

    test = tests[1][2].pop()
    test_obj = wpttest.from_manifest(tests, test, [], test_metadata.get_test(test.id))
    assert test_obj.prefs == {"a": "b", "c": "d"}
    assert test_obj.expected() == "FAIL"


def test_metadata_lsan_stack_depth():
    tests = make_mock_manifest(("test", "a", 10), ("test", "a/b", 10))

    test_metadata = manifestexpected.static.compile(BytesIO(test_2),
                                                    {},
                                                    data_cls_getter=manifestexpected.data_cls_getter,
                                                    test_path="a",
                                                    url_base="")

    test = tests[2][2].pop()
    test_obj = wpttest.from_manifest(tests, test, [], test_metadata.get_test(test.id))

    assert test_obj.lsan_max_stack_depth == 42

    test = tests[1][2].pop()
    test_obj = wpttest.from_manifest(tests, test, [], test_metadata.get_test(test.id))

    assert test_obj.lsan_max_stack_depth is None

    test_metadata = manifestexpected.static.compile(BytesIO(test_0),
                                                    {},
                                                    data_cls_getter=manifestexpected.data_cls_getter,
                                                    test_path="a",
                                                    url_base="")

    inherit_metadata = [
        manifestexpected.static.compile(
            BytesIO(dir_ini_2),
            {},
            data_cls_getter=lambda x,y: manifestexpected.DirectoryManifest)
    ]

    test = tests[0][2].pop()
    test_obj = wpttest.from_manifest(tests, test, inherit_metadata, test_metadata.get_test(test.id))

    assert test_obj.lsan_max_stack_depth == 42
