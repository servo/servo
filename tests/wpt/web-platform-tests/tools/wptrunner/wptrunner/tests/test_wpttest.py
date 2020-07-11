import mock
from io import BytesIO

from manifest import manifest as wptmanifest
from manifest.item import TestharnessTest, RefTest
from manifest.utils import to_os_path
from . test_update import tree_and_sourcefile_mocks
from .. import manifestexpected, wpttest


dir_ini_0 = b"""\
prefs: [a:b]
"""

dir_ini_1 = b"""\
prefs: [@Reset, b:c]
max-asserts: 2
min-asserts: 1
tags: [b, c]
"""

dir_ini_2 = b"""\
lsan-max-stack-depth: 42
"""

test_0 = b"""\
[0.html]
  prefs: [c:d]
  max-asserts: 3
  tags: [a, @Reset]
"""

test_1 = b"""\
[1.html]
  prefs:
    if os == 'win': [a:b, c:d]
  expected:
    if os == 'win': FAIL
"""

test_2 = b"""\
[2.html]
  lsan-max-stack-depth: 42
"""

test_3 = b"""\
[3.html]
  [subtest1]
    expected: [PASS, FAIL]

  [subtest2]
    disabled: reason

  [subtest3]
    expected: FAIL
"""

test_4 = b"""\
[4.html]
  expected: FAIL
"""

test_5 = b"""\
[5.html]
"""

test_6 = b"""\
[6.html]
  expected: [OK, FAIL]
"""

test_7 = b"""\
[7.html]
  blink_expect_any_subtest_status: yep
"""

test_fuzzy = b"""\
[fuzzy.html]
  fuzzy: fuzzy-ref.html:1;200
"""


testharness_test = b"""<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>"""


def make_mock_manifest(*items):
    rv = mock.Mock(tests_root="/foobar")
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


def test_conditional():
    items = [("test", "a", 10), ("test", "a/b", 10), ("test", "c", 10)]

    test_obj = make_test_object(test_1, "a/1.html", 1, items, None, True, {"os": "win"})

    assert test_obj.prefs == {"a": "b", "c": "d"}
    assert test_obj.expected() == "FAIL"


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


def test_subtests():
    test_obj = make_test_object(test_3, "a/3.html", 3, ("test", "a", 4), None, False)
    assert test_obj.expected("subtest1") == "PASS"
    assert test_obj.known_intermittent("subtest1") == ["FAIL"]
    assert test_obj.expected("subtest2") == "PASS"
    assert test_obj.known_intermittent("subtest2") == []
    assert test_obj.expected("subtest3") == "FAIL"
    assert test_obj.known_intermittent("subtest3") == []


def test_expected_fail():
    test_obj = make_test_object(test_4, "a/4.html", 4, ("test", "a", 5), None, False)
    assert test_obj.expected() == "FAIL"
    assert test_obj.known_intermittent() == []


def test_no_expected():
    test_obj = make_test_object(test_5, "a/5.html", 5, ("test", "a", 6), None, False)
    assert test_obj.expected() == "OK"
    assert test_obj.known_intermittent() == []


def test_known_intermittent():
    test_obj = make_test_object(test_6, "a/6.html", 6, ("test", "a", 7), None, False)
    assert test_obj.expected() == "OK"
    assert test_obj.known_intermittent() == ["FAIL"]


def test_expect_any_subtest_status():
    test_obj = make_test_object(test_7, "a/7.html", 7, ("test", "a", 8), None, False)
    assert test_obj.expected() == "OK"
    assert test_obj.expect_any_subtest_status() is True


def test_metadata_fuzzy():
    item = RefTest(".", "a/fuzzy.html", "/", "a/fuzzy.html",
                   references=[["/a/fuzzy-ref.html", "=="]],
                   fuzzy=[[["/a/fuzzy.html", '/a/fuzzy-ref.html', '=='],
                           [[2, 3], [10, 15]]]])
    s = mock.Mock(rel_path="a/fuzzy.html", rel_path_parts=("a", "fuzzy.html"), hash="0"*40)
    s.manifest_items = mock.Mock(return_value=(item.item_type, [item]))


    manifest = wptmanifest.Manifest("")

    tree, sourcefile_mock = tree_and_sourcefile_mocks([(s, None, True)])
    with mock.patch("manifest.manifest.SourceFile", side_effect=sourcefile_mock):
        assert manifest.update(tree) is True

    test_metadata = manifestexpected.static.compile(BytesIO(test_fuzzy),
                                                    {},
                                                    data_cls_getter=manifestexpected.data_cls_getter,
                                                    test_path="a/fuzzy.html",
                                                    url_base="/")

    test = next(manifest.iterpath(to_os_path("a/fuzzy.html")))
    test_obj = wpttest.from_manifest(manifest, test, [], test_metadata.get_test(test.id))

    assert test_obj.fuzzy == {('/a/fuzzy.html', '/a/fuzzy-ref.html', '=='): [[2, 3], [10, 15]]}
    assert test_obj.fuzzy_override == {'/a/fuzzy-ref.html': ((1, 1), (200, 200))}
