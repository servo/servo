import pytest
import sys
from io import BytesIO
from mock import Mock

from manifest import manifest as wptmanifest
from manifest.item import TestharnessTest
from .. import manifestexpected, wpttest

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

test_3 = """\
[3.html]
  [subtest1]
    expected: [PASS, FAIL]

  [subtest2]
    disabled: reason

  [subtest3]
    expected: FAIL
"""

test_4 = """\
[4.html]
  expected: FAIL
"""

test_5 = """\
[5.html]
"""

test_6 = """\
[6.html]
  expected: [OK, FAIL]
"""

test_fuzzy = """\
[fuzzy.html]
  fuzzy: fuzzy-ref.html:1;200
"""


testharness_test = """<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>"""


def make_mock_manifest(*items):
    rv = Mock(tests_root="/foobar")
    tests = []
    rv.__iter__ = lambda self: iter(tests)
    rv.__getitem__ = lambda self, k: tests[k]
    for test_type, dir_path, num_tests in items:
        for i in range(num_tests):
            filename = dir_path + "/%i.html" % i
            tests.append((test_type,
                          filename,
                          {TestharnessTest("/foo.bar", filename, "/", filename)}))
    return rv

def make_test_object(test_name,
                     test_path,
                     index,
                     items,
                     inherit_metadata=None,
                     iterate=False,
                     condition=None):
    inherit_metadata = inherit_metadata if inherit_metadata is not None else []
    condition = condition if condition is not None else {}
    tests = make_mock_manifest(*items) if isinstance(items, list) else make_mock_manifest(items)

    test_metadata = manifestexpected.static.compile(BytesIO(test_name),
                                                    condition,
                                                    data_cls_getter=manifestexpected.data_cls_getter,
                                                    test_path=test_path,
                                                    url_base="/")

    test = next(iter(tests[index][2])) if iterate else tests[index][2].pop()
    return wpttest.from_manifest(tests, test, inherit_metadata, test_metadata.get_test(test.id))


@pytest.mark.xfail(sys.version[0] == "3",
                   reason="bytes/text confusion in py3")
def test_metadata_inherit():
    items = [("test", "a", 10), ("test", "a/b", 10), ("test", "c", 10)]
    inherit_metadata = [
        manifestexpected.static.compile(
            BytesIO(item),
            {},
            data_cls_getter=lambda x,y: manifestexpected.DirectoryManifest)
        for item in [dir_ini_0, dir_ini_1]]

    test_obj = make_test_object(test_0, "a/0.html", 0, items, inherit_metadata, True)

    assert test_obj.max_assertion_count == 3
    assert test_obj.min_assertion_count == 1
    assert test_obj.prefs == {"b": "c", "c": "d"}
    assert test_obj.tags == {"a", "dir:a"}


@pytest.mark.xfail(sys.version[0] == "3",
                   reason="bytes/text confusion in py3")
def test_conditional():
    items = [("test", "a", 10), ("test", "a/b", 10), ("test", "c", 10)]

    test_obj = make_test_object(test_1, "a/1.html", 1, items, None, True, {"os": "win"})

    assert test_obj.prefs == {"a": "b", "c": "d"}
    assert test_obj.expected() == "FAIL"


@pytest.mark.xfail(sys.version[0] == "3",
                   reason="bytes/text confusion in py3")
def test_metadata_lsan_stack_depth():
    items = [("test", "a", 10), ("test", "a/b", 10)]

    test_obj = make_test_object(test_2, "a/2.html", 2, items, None, True)

    assert test_obj.lsan_max_stack_depth == 42

    test_obj = make_test_object(test_2, "a/2.html", 1, items, None, True)

    assert test_obj.lsan_max_stack_depth is None

    inherit_metadata = [
        manifestexpected.static.compile(
            BytesIO(dir_ini_2),
            {},
            data_cls_getter=lambda x,y: manifestexpected.DirectoryManifest)
    ]

    test_obj = make_test_object(test_0, "a/0/html", 0, items, inherit_metadata, False)

    assert test_obj.lsan_max_stack_depth == 42


@pytest.mark.xfail(sys.version[0] == "3",
                   reason="bytes/text confusion in py3")
def test_subtests():
    test_obj = make_test_object(test_3, "a/3.html", 3, ("test", "a", 4), None, False)
    assert test_obj.expected("subtest1") == "PASS"
    assert test_obj.known_intermittent("subtest1") == ["FAIL"]
    assert test_obj.expected("subtest2") == "PASS"
    assert test_obj.known_intermittent("subtest2") == []
    assert test_obj.expected("subtest3") == "FAIL"
    assert test_obj.known_intermittent("subtest3") == []


@pytest.mark.xfail(sys.version[0] == "3",
                   reason="bytes/text confusion in py3")
def test_expected_fail():
    test_obj = make_test_object(test_4, "a/4.html", 4, ("test", "a", 5), None, False)
    assert test_obj.expected() == "FAIL"
    assert test_obj.known_intermittent() == []


@pytest.mark.xfail(sys.version[0] == "3",
                   reason="bytes/text confusion in py3")
def test_no_expected():
    test_obj = make_test_object(test_5, "a/5.html", 5, ("test", "a", 6), None, False)
    assert test_obj.expected() == "OK"
    assert test_obj.known_intermittent() == []


@pytest.mark.xfail(sys.version[0] == "3",
                   reason="bytes/text confusion in py3")
def test_known_intermittent():
    test_obj = make_test_object(test_6, "a/6.html", 6, ("test", "a", 7), None, False)
    assert test_obj.expected() == "OK"
    assert test_obj.known_intermittent() == ["FAIL"]


@pytest.mark.xfail(sys.version[0] == "3",
                   reason="bytes/text confusion in py3")
def test_metadata_fuzzy():
    manifest_data = {
        "items": {"reftest": {"a/fuzzy.html": [["a/fuzzy.html",
                                                [["/a/fuzzy-ref.html", "=="]],
                                                {"fuzzy": [[["/a/fuzzy.html", '/a/fuzzy-ref.html', '=='],
                                                            [[2, 3], [10, 15]]]]}]]}},
        "paths": {"a/fuzzy.html": ["0"*40, "reftest"]},
        "version": 6,
        "url_base": "/"}
    manifest = wptmanifest.Manifest.from_json(".", manifest_data)
    test_metadata = manifestexpected.static.compile(BytesIO(test_fuzzy),
                                                    {},
                                                    data_cls_getter=manifestexpected.data_cls_getter,
                                                    test_path="a/fuzzy.html",
                                                    url_base="/")

    test = manifest.iterpath("a/fuzzy.html").next()
    test_obj = wpttest.from_manifest(manifest, test, [], test_metadata.get_test(test.id))

    assert test_obj.fuzzy == {('/a/fuzzy.html', '/a/fuzzy-ref.html', '=='): [[2, 3], [10, 15]]}
    assert test_obj.fuzzy_override == {'/a/fuzzy-ref.html': ((1, 1), (200, 200))}
