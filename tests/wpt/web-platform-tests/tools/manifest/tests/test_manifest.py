import os

import mock

import hypothesis as h
import hypothesis.strategies as hs

import pytest

from .. import manifest, sourcefile, item, utils

MYPY = False
if MYPY:
    # MYPY is set to True when run under Mypy.
    from typing import Any
    from typing import Type


def SourceFileWithTest(path, hash, cls, **kwargs):
    # type: (str, str, Type[item.ManifestItem], **Any) -> sourcefile.SourceFile
    s = mock.Mock(rel_path=path, hash=hash)
    if cls == item.SupportFile:
        test = cls("/foobar", path)
    else:
        assert issubclass(cls, item.URLManifestItem)
        test = cls("/foobar", path, "/", utils.from_os_path(path), **kwargs)
    s.manifest_items = mock.Mock(return_value=(cls.item_type, [test]))
    return s  # type: ignore

def SourceFileWithTests(path, hash, cls, variants):
    # type: (str, str, Type[item.URLManifestItem], **Any) -> sourcefile.SourceFile
    s = mock.Mock(rel_path=path, hash=hash)
    tests = [cls("/foobar", path, "/", item[0], **item[1]) for item in variants]
    s.manifest_items = mock.Mock(return_value=(cls.item_type, tests))
    return s  # type: ignore


@hs.composite
def rel_dir_file_path(draw):
    length = draw(hs.integers(min_value=1, max_value=20))
    if length == 1:
        return "a"
    else:
        remaining = length - 2
        alphabet = "a" + os.path.sep
        mid = draw(hs.text(alphabet=alphabet, min_size=remaining, max_size=remaining))
        return os.path.normcase("a" + mid + "a")


@hs.composite
def sourcefile_strategy(draw):
    item_classes = [item.TestharnessTest, item.RefTestNode,
                    item.ManualTest, item.WebDriverSpecTest,
                    item.ConformanceCheckerTest, item.SupportFile]
    cls = draw(hs.sampled_from(item_classes))

    path = draw(rel_dir_file_path())
    hash = draw(hs.text(alphabet="0123456789abcdef", min_size=40, max_size=40))
    s = mock.Mock(rel_path=path, hash=hash)

    if cls is item.RefTestNode:
        ref_path = draw(rel_dir_file_path())
        h.assume(path != ref_path)
        ref_eq = draw(hs.sampled_from(["==", "!="]))
        test = cls("/foobar", path, "/", utils.from_os_path(path), references=[(utils.from_os_path(ref_path), ref_eq)])
    elif cls is item.SupportFile:
        test = cls("/foobar", path)
    else:
        test = cls("/foobar", path, "/", utils.from_os_path(path))

    s.manifest_items = mock.Mock(return_value=(cls.item_type, [test]))
    return s


@h.given(hs.lists(sourcefile_strategy(),
                  min_size=1, max_size=1000, unique_by=lambda x: x.rel_path))
@h.example([SourceFileWithTest("a", "0"*40, item.ConformanceCheckerTest)])
def test_manifest_to_json(s):
    m = manifest.Manifest()

    assert m.update((item, True) for item in s) is True

    json_str = m.to_json()
    loaded = manifest.Manifest.from_json("/", json_str)

    assert list(loaded) == list(m)

    assert loaded.to_json() == json_str


@h.given(hs.lists(sourcefile_strategy(),
                  min_size=1, unique_by=lambda x: x.rel_path))
@h.example([SourceFileWithTest("a", "0"*40, item.TestharnessTest)])
@h.example([SourceFileWithTest("a", "0"*40, item.RefTestNode, references=[("/aa", "==")])])
def test_manifest_idempotent(s):
    m = manifest.Manifest()

    assert m.update((item, True) for item in s) is True

    m1 = list(m)

    assert m.update((item, True) for item in s) is False

    assert list(m) == m1


def test_manifest_to_json_forwardslash():
    m = manifest.Manifest()

    s = SourceFileWithTest("a/b", "0"*40, item.TestharnessTest)

    assert m.update([(s, True)]) is True

    assert m.to_json() == {
        'paths': {
            'a/b': ('0000000000000000000000000000000000000000', 'testharness')
        },
        'version': 7,
        'url_base': '/',
        'items': {
            'testharness': {
                'a/b': [('a/b', {})]
            }
        }
    }


@pytest.mark.skipif(os.sep != "\\", reason="backslash path")
def test_manifest_to_json_backslash():
    m = manifest.Manifest()

    s = SourceFileWithTest("a\\b", "0"*40, item.TestharnessTest)

    assert m.update([(s, True)]) is True

    assert m.to_json() == {
        'paths': {
            'a/b': ('0000000000000000000000000000000000000000', 'testharness')
        },
        'version': 7,
        'url_base': '/',
        'items': {
            'testharness': {
                'a/b': [('a/b', {})]
            }
        }
    }


def test_manifest_from_json_backslash():
    json_obj = {
        'paths': {
            'a\\b': ('0000000000000000000000000000000000000000', 'testharness')
        },
        'version': 7,
        'url_base': '/',
        'items': {
            'testharness': {
                'a\\b': [['a/b', {}]]
            }
        }
    }

    with pytest.raises(ValueError):
        manifest.Manifest.from_json("/", json_obj)


def test_reftest_computation_chain():
    m = manifest.Manifest()

    s1 = SourceFileWithTest("test1", "0"*40, item.RefTestNode, references=[("/test2", "==")])
    s2 = SourceFileWithTest("test2", "0"*40, item.RefTestNode, references=[("/test3", "==")])

    m.update([(s1, True), (s2, True)])

    test1 = s1.manifest_items()[1][0]
    test2 = s2.manifest_items()[1][0]

    assert list(m) == [("reftest", test1.path, {test1.to_RefTest()}),
                       ("reftest_node", test2.path, {test2})]


def test_reftest_computation_chain_update_add():
    m = manifest.Manifest()

    s2 = SourceFileWithTest("test2", "0"*40, item.RefTestNode, references=[("/test3", "==")])
    test2 = s2.manifest_items()[1][0]

    assert m.update([(s2, True)]) is True

    assert list(m) == [("reftest", test2.path, {test2.to_RefTest()})]

    s1 = SourceFileWithTest("test1", "0"*40, item.RefTestNode, references=[("/test2", "==")])
    test1 = s1.manifest_items()[1][0]

    # s2's hash is unchanged, but it has gone from a test to a node
    assert m.update([(s1, True), (s2, True)]) is True

    assert list(m) == [("reftest", test1.path, {test1.to_RefTest()}),
                       ("reftest_node", test2.path, {test2})]


def test_reftest_computation_chain_update_remove():
    m = manifest.Manifest()

    s1 = SourceFileWithTest("test1", "0"*40, item.RefTestNode, references=[("/test2", "==")])
    s2 = SourceFileWithTest("test2", "0"*40, item.RefTestNode, references=[("/test3", "==")])

    assert m.update([(s1, True), (s2, True)]) is True

    test1 = s1.manifest_items()[1][0]
    test2 = s2.manifest_items()[1][0]

    assert list(m) == [("reftest", test1.path, {test1.to_RefTest()}),
                       ("reftest_node", test2.path, {test2})]

    # s2's hash is unchanged, but it has gone from a node to a test
    assert m.update([(s2, True)]) is True

    assert list(m) == [("reftest", test2.path, {test2.to_RefTest()})]


def test_reftest_computation_chain_update_test_type():
    m = manifest.Manifest()

    s1 = SourceFileWithTest("test", "0"*40, item.RefTestNode, references=[("/test-ref", "==")])

    assert m.update([(s1, True)]) is True

    test1 = s1.manifest_items()[1][0]

    assert list(m) == [("reftest", test1.path, {test1.to_RefTest()})]

    # test becomes a testharness test (hash change because that is determined
    # based on the file contents). The updated manifest should not includes the
    # old reftest.
    s2 = SourceFileWithTest("test", "1"*40, item.TestharnessTest)
    assert m.update([(s2, True)]) is True

    test2 = s2.manifest_items()[1][0]

    assert list(m) == [("testharness", test2.path, {test2})]


def test_reftest_computation_chain_update_node_change():
    m = manifest.Manifest()

    s1 = SourceFileWithTest("test1", "0"*40, item.RefTestNode, references=[("/test2", "==")])
    s2 = SourceFileWithTest("test2", "0"*40, item.RefTestNode, references=[("/test3", "==")])

    assert m.update([(s1, True), (s2, True)]) is True

    test1 = s1.manifest_items()[1][0]
    test2 = s2.manifest_items()[1][0]

    assert list(m) == [("reftest", test1.path, {test1.to_RefTest()}),
                       ("reftest_node", test2.path, {test2})]

    #test2 changes to support type
    s2 = SourceFileWithTest("test2", "1"*40, item.SupportFile)

    assert m.update([(s1, True), (s2, True)]) is True
    test3 = s2.manifest_items()[1][0]

    assert list(m) == [("reftest", test1.path, {test1.to_RefTest()}),
                       ("support", test3.path, {test3})]


def test_reftest_computation_chain_update_node_change_partial():
    m = manifest.Manifest()

    s1 = SourceFileWithTest("test1", "0"*40, item.RefTestNode, references=[("/test2", "==")])
    s2 = SourceFileWithTest("test2", "0"*40, item.RefTestNode, references=[("/test3", "==")])

    assert m.update([(s1, True), (s2, True)]) is True

    test1 = s1.manifest_items()[1][0]
    test2 = s2.manifest_items()[1][0]

    assert list(m) == [("reftest", test1.path, {test1.to_RefTest()}),
                       ("reftest_node", test2.path, {test2})]

    s2 = SourceFileWithTest("test2", "1"*40, item.RefTestNode, references=[("/test3", "==")])

    assert m.update([(s1.rel_path, False), (s2, True)]) is True

    assert list(m) == [("reftest", test1.path, {test1.to_RefTest()}),
                       ("reftest_node", test2.path, {test2})]


def test_iterpath():
    m = manifest.Manifest()

    sources = [SourceFileWithTest("test1", "0"*40, item.RefTestNode, references=[("/test1-ref", "==")]),
               SourceFileWithTests("test2", "1"*40, item.TestharnessTest, [("test2-1.html", {}),
                                                                           ("test2-2.html", {})]),
               SourceFileWithTest("test3", "0"*40, item.TestharnessTest)]
    m.update([(s, True) for s in sources])

    assert {item.url for item in m.iterpath("test2")} == {"/test2-1.html",
                                                          "/test2-2.html"}
    assert set(m.iterpath("missing")) == set()


def test_reftest_node_by_url():
    m = manifest.Manifest()

    s1 = SourceFileWithTest("test1", "0"*40, item.RefTestNode, references=[("/test2", "==")])
    s2 = SourceFileWithTest("test2", "0"*40, item.RefTestNode, references=[("/test3", "==")])

    m.update([(s1, True), (s2, True)])

    test1 = s1.manifest_items()[1][0]
    test2 = s2.manifest_items()[1][0]

    assert m.reftest_nodes_by_url == {"/test1": test1.to_RefTest(),
                                      "/test2": test2}
    m._reftest_nodes_by_url = None
    assert m.reftest_nodes_by_url == {"/test1": test1.to_RefTest(),
                                      "/test2": test2}


def test_no_update():
    m = manifest.Manifest()

    s1 = SourceFileWithTest("test1", "0"*40, item.TestharnessTest)
    s2 = SourceFileWithTest("test2", "0"*40, item.TestharnessTest)

    m.update([(s1, True), (s2, True)])

    test1 = s1.manifest_items()[1][0]
    test2 = s2.manifest_items()[1][0]

    assert list(m) == [("testharness", test1.path, {test1}),
                       ("testharness", test2.path, {test2})]

    s1_1 = SourceFileWithTest("test1", "1"*40, item.ManualTest)

    m.update([(s1_1, True), (s2.rel_path, False)])

    test1_1 = s1_1.manifest_items()[1][0]

    assert list(m) == [("manual", test1_1.path, {test1_1}),
                       ("testharness", test2.path, {test2})]


def test_no_update_delete():
    m = manifest.Manifest()

    s1 = SourceFileWithTest("test1", "0"*40, item.TestharnessTest)
    s2 = SourceFileWithTest("test2", "0"*40, item.TestharnessTest)

    m.update([(s1, True), (s2, True)])

    test1 = s1.manifest_items()[1][0]

    s1_1 = SourceFileWithTest("test1", "1"*40, item.ManualTest)

    m.update([(s1_1.rel_path, False)])

    assert list(m) == [("testharness", test1.path, {test1})]


def test_update_from_json():
    m = manifest.Manifest()

    s1 = SourceFileWithTest("test1", "0"*40, item.TestharnessTest)
    s2 = SourceFileWithTest("test2", "0"*40, item.TestharnessTest)

    m.update([(s1, True), (s2, True)])

    json_str = m.to_json()
    m = manifest.Manifest.from_json("/", json_str)

    m.update([(s1, True)])

    test1 = s1.manifest_items()[1][0]

    assert list(m) == [("testharness", test1.path, {test1})]


def test_update_from_json_modified():
    # Create the original manifest
    m = manifest.Manifest()
    s1 = SourceFileWithTest("test1", "0"*40, item.TestharnessTest)
    m.update([(s1, True)])
    json_str = m.to_json()

    # Reload it from JSON
    m = manifest.Manifest.from_json("/", json_str)

    # Update it with timeout="long"
    s2 = SourceFileWithTest("test1", "1"*40, item.TestharnessTest, timeout="long")
    m.update([(s2, True)])
    json_str = m.to_json()
    assert json_str == {
        'items': {'testharness': {'test1': [('test1', {"timeout": "long"})]}},
        'paths': {'test1': ('1111111111111111111111111111111111111111',
                            'testharness')},
        'url_base': '/',
        'version': 7
    }
