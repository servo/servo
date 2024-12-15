# mypy: ignore-errors

import os
import sys
from unittest import mock

import hypothesis as h
import hypothesis.strategies as hs
import pytest

from .. import manifest, sourcefile, item, utils

from typing import Any, Type


def SourceFileWithTest(path: str, hash: str, cls: Type[item.ManifestItem], **kwargs: Any) -> sourcefile.SourceFile:
    rel_path_parts = tuple(path.split(os.path.sep))
    s = mock.Mock(rel_path=path,
                  rel_path_parts=rel_path_parts,
                  hash=hash)
    if cls == item.SupportFile:
        test = cls("/foobar", path)
    else:
        assert issubclass(cls, item.URLManifestItem)
        test = cls("/foobar", path, "/", utils.from_os_path(path), **kwargs)
    s.manifest_items = mock.Mock(return_value=(cls.item_type, [test]))
    return s  # type: ignore


def SourceFileWithTests(path: str, hash: str, cls: Type[item.URLManifestItem], variants: Any) -> sourcefile.SourceFile:
    rel_path_parts = tuple(path.split(os.path.sep))
    s = mock.Mock(rel_path=path,
                  rel_path_parts=rel_path_parts,
                  hash=hash)
    tests = [cls("/foobar", path, "/", item[0], **item[1]) for item in variants]
    s.manifest_items = mock.Mock(return_value=(cls.item_type, tests))
    return s  # type: ignore


def tree_and_sourcefile_mocks(source_files):
    paths_dict = {}
    tree = []
    for source_file, file_hash, updated in source_files:
        paths_dict[source_file.rel_path] = source_file
        tree.append([source_file.rel_path, file_hash, updated])

    def MockSourceFile(tests_root, path, url_base, file_hash):
        return paths_dict[path]

    return tree, MockSourceFile


@hs.composite
def sourcefile_strategy(draw):
    item_classes = [item.TestharnessTest, item.RefTest, item.PrintRefTest,
                    item.ManualTest, item.WebDriverSpecTest,
                    item.ConformanceCheckerTest, item.SupportFile]
    cls = draw(hs.sampled_from(item_classes))

    path = "a"
    rel_path_parts = tuple(path.split(os.path.sep))
    hash = draw(hs.text(alphabet="0123456789abcdef", min_size=40, max_size=40))
    s = mock.Mock(rel_path=path,
                  rel_path_parts=rel_path_parts,
                  hash=hash)

    if cls in (item.RefTest, item.PrintRefTest):
        ref_path = "b"
        ref_eq = draw(hs.sampled_from(["==", "!="]))
        test = cls("/foobar", path, "/", utils.from_os_path(path), references=[(utils.from_os_path(ref_path), ref_eq)])
    elif cls is item.SupportFile:
        test = cls("/foobar", path)
    else:
        test = cls("/foobar", path, "/", "foobar")

    s.manifest_items = mock.Mock(return_value=(cls.item_type, [test]))
    return s


@hs.composite
def manifest_tree(draw):
    names = hs.text(alphabet=hs.characters(blacklist_characters="\0/\\:*\"?<>|"), min_size=1)
    tree = hs.recursive(sourcefile_strategy(),
                        lambda children: hs.dictionaries(names, children, min_size=1),
                        max_leaves=10)

    generated_root = draw(tree)
    h.assume(isinstance(generated_root, dict))

    reftest_urls = []
    output = []
    stack = [((k,), v) for k, v in generated_root.items()]
    while stack:
        path, node = stack.pop()
        if isinstance(node, dict):
            stack.extend((path + (k,), v) for k, v in node.items())
        else:
            rel_path = os.path.sep.join(path)
            node.rel_path = rel_path
            node.rel_path_parts = tuple(path)
            for test_item in node.manifest_items.return_value[1]:
                test_item.path = rel_path
                if isinstance(test_item, item.RefTest):
                    if reftest_urls:
                        possible_urls = hs.sampled_from(reftest_urls) | names
                    else:
                        possible_urls = names
                    reference = hs.tuples(hs.sampled_from(["==", "!="]),
                                          possible_urls)
                    references = hs.lists(reference, min_size=1, unique=True)
                    test_item.references = draw(references)
                    reftest_urls.append(test_item.url)
            output.append(node)

    return output


@pytest.mark.skipif(sys.version_info[:3] in ((3, 10, 10), (3, 11, 2)),
                    reason="https://github.com/python/cpython/issues/102126")
@h.given(manifest_tree())
# FIXME: Workaround for https://github.com/web-platform-tests/wpt/issues/22758
@h.settings(suppress_health_check=(h.HealthCheck.too_slow,))
@h.example([SourceFileWithTest("a", "0"*40, item.ConformanceCheckerTest)])
def test_manifest_to_json(s):
    m = manifest.Manifest("")

    tree, sourcefile_mock = tree_and_sourcefile_mocks((item, None, True) for item in s)
    with mock.patch("tools.manifest.manifest.SourceFile", side_effect=sourcefile_mock):
        assert m.update(tree) is True

    json_str = m.to_json()
    loaded = manifest.Manifest.from_json("/", json_str)

    assert list(loaded) == list(m)

    assert loaded.to_json() == json_str


@pytest.mark.skipif(sys.version_info[:3] in ((3, 10, 10), (3, 11, 2)),
                    reason="https://github.com/python/cpython/issues/102126")
@h.given(manifest_tree())
# FIXME: Workaround for https://github.com/web-platform-tests/wpt/issues/22758
@h.settings(suppress_health_check=(h.HealthCheck.too_slow,))
@h.example([SourceFileWithTest("a", "0"*40, item.TestharnessTest)])
@h.example([SourceFileWithTest("a", "0"*40, item.RefTest, references=[("/aa", "==")])])
def test_manifest_idempotent(s):
    m = manifest.Manifest("")

    tree, sourcefile_mock = tree_and_sourcefile_mocks((item, None, True) for item in s)
    with mock.patch("tools.manifest.manifest.SourceFile", side_effect=sourcefile_mock):
        assert m.update(tree) is True

    m1 = list(m)

    with mock.patch("tools.manifest.manifest.SourceFile", side_effect=sourcefile_mock):
        assert m.update(tree) is False

    assert list(m) == m1


def test_manifest_to_json_forwardslash():
    m = manifest.Manifest("")

    s = SourceFileWithTest("a" + os.path.sep + "b", "0"*40, item.TestharnessTest)

    tree, sourcefile_mock = tree_and_sourcefile_mocks([(s, None, True)])
    with mock.patch("tools.manifest.manifest.SourceFile", side_effect=sourcefile_mock):
        assert m.update(tree) is True

    assert m.to_json() == {
        'version': 9,
        'url_base': '/',
        'items': {
            'testharness': {'a': {'b': [
                '0000000000000000000000000000000000000000',
                (None, {})
            ]}},
        }
    }


def test_reftest_computation_chain():
    m = manifest.Manifest("")

    s1 = SourceFileWithTest("test1", "0"*40, item.RefTest, references=[("/test2", "==")])
    s2 = SourceFileWithTest("test2", "0"*40, item.RefTest, references=[("/test3", "==")])

    tree, sourcefile_mock = tree_and_sourcefile_mocks([(s1, None, True), (s2, None, True)])
    with mock.patch("tools.manifest.manifest.SourceFile", side_effect=sourcefile_mock):
        m.update(tree)

    test1 = s1.manifest_items()[1][0]
    test2 = s2.manifest_items()[1][0]

    assert list(m) == [("reftest", test1.path, {test1}),
                       ("reftest", test2.path, {test2})]


def test_iterpath():
    m = manifest.Manifest("")

    sources = [SourceFileWithTest("test1", "0"*40, item.RefTest, references=[("/test1-ref", "==")]),
               SourceFileWithTests("test2", "1"*40, item.TestharnessTest, [("test2-1.html", {}),
                                                                           ("test2-2.html", {})]),
               SourceFileWithTest("test3", "0"*40, item.TestharnessTest)]
    tree, sourcefile_mock = tree_and_sourcefile_mocks((item, None, True) for item in sources)
    assert len(tree) == len(sources)
    with mock.patch("tools.manifest.manifest.SourceFile", side_effect=sourcefile_mock):
        m.update(tree)

    assert {item.url for item in m.iterpath("test2")} == {"/test2-1.html",
                                                          "/test2-2.html"}
    assert set(m.iterpath("missing")) == set()


def test_no_update():
    m = manifest.Manifest("")

    s1 = SourceFileWithTest("test1", "0"*40, item.TestharnessTest)
    s2 = SourceFileWithTest("test2", "0"*40, item.TestharnessTest)

    tree, sourcefile_mock = tree_and_sourcefile_mocks((item, None, True) for item in [s1, s2])
    with mock.patch("tools.manifest.manifest.SourceFile", side_effect=sourcefile_mock):
        m.update(tree)

    test1 = s1.manifest_items()[1][0]
    test2 = s2.manifest_items()[1][0]

    assert list(m) == [("testharness", test1.path, {test1}),
                       ("testharness", test2.path, {test2})]

    s1_1 = SourceFileWithTest("test1", "1"*40, item.ManualTest)

    tree, sourcefile_mock = tree_and_sourcefile_mocks([(s1_1, None, True), (s2, None, False)])
    with mock.patch("tools.manifest.manifest.SourceFile", side_effect=sourcefile_mock):
        m.update(tree)

    test1_1 = s1_1.manifest_items()[1][0]

    assert list(m) == [("manual", test1_1.path, {test1_1}),
                       ("testharness", test2.path, {test2})]


def test_no_update_delete():
    m = manifest.Manifest("")

    s1 = SourceFileWithTest("test1", "0"*40, item.TestharnessTest)
    s2 = SourceFileWithTest("test2", "0"*40, item.TestharnessTest)

    tree, sourcefile_mock = tree_and_sourcefile_mocks([(s1, None, True), (s2, None, True)])
    with mock.patch("tools.manifest.manifest.SourceFile", side_effect=sourcefile_mock):
        m.update(tree)

    test1 = s1.manifest_items()[1][0]

    tree, sourcefile_mock = tree_and_sourcefile_mocks([(s1, None, False)])
    with mock.patch("tools.manifest.manifest.SourceFile", side_effect=sourcefile_mock):
        m.update(tree)

    assert list(m) == [("testharness", test1.path, {test1})]


def test_update_from_json():
    m = manifest.Manifest("")

    s1 = SourceFileWithTest("test1", "0"*40, item.TestharnessTest)
    s2 = SourceFileWithTest("test2", "0"*40, item.TestharnessTest)

    tree, sourcefile_mock = tree_and_sourcefile_mocks([(s1, None, True), (s2, None, True)])
    with mock.patch("tools.manifest.manifest.SourceFile", side_effect=sourcefile_mock):
        m.update(tree)

    json_str = m.to_json()
    m = manifest.Manifest.from_json("/", json_str)

    tree, sourcefile_mock = tree_and_sourcefile_mocks([(s1, None, True)])
    with mock.patch("tools.manifest.manifest.SourceFile", side_effect=sourcefile_mock):
        m.update(tree)

    test1 = s1.manifest_items()[1][0]

    assert list(m) == [("testharness", test1.path, {test1})]


def test_update_from_json_modified():
    # Create the original manifest
    m = manifest.Manifest("")
    s1 = SourceFileWithTest("test1", "0"*40, item.TestharnessTest)
    tree, sourcefile_mock = tree_and_sourcefile_mocks([(s1, None, True)])
    with mock.patch("tools.manifest.manifest.SourceFile", side_effect=sourcefile_mock):
        m.update(tree)
    json_str = m.to_json()

    # Reload it from JSON
    m = manifest.Manifest.from_json("/", json_str)

    # Update it with timeout="long"
    s2 = SourceFileWithTest("test1", "1"*40, item.TestharnessTest, timeout="long", pac="proxy.pac")
    tree, sourcefile_mock = tree_and_sourcefile_mocks([(s2, None, True)])
    with mock.patch("tools.manifest.manifest.SourceFile", side_effect=sourcefile_mock):
        m.update(tree)
    json_str = m.to_json()
    assert json_str == {
        'items': {'testharness': {'test1': [
            "1"*40,
            (None, {'timeout': 'long', 'pac': 'proxy.pac'})
        ]}},
        'url_base': '/',
        'version': 9
    }

def test_manifest_spec_to_json():
    m = manifest.Manifest("")

    path = "a" + os.path.sep + "b"
    hash = "0"*40
    rel_path_parts = tuple(path.split(os.path.sep))
    s = mock.Mock(rel_path=path,
                  rel_path_parts=rel_path_parts,
                  hash=hash)
    spec = item.SpecItem("/foobar", path, ["specA"])
    s.manifest_spec_items = mock.Mock(return_value=(item.SpecItem.item_type, [spec]))

    tree, sourcefile_mock = tree_and_sourcefile_mocks([(s, None, True)])
    with mock.patch("tools.manifest.manifest.SourceFile", side_effect=sourcefile_mock):
        assert m.update(tree, True, manifest.compute_manifest_spec_items) is True

    assert m.to_json() == {
        'version': 9,
        'url_base': '/',
        'items': {
            'spec': {'a': {'b': [
                '0000000000000000000000000000000000000000',
                (None, {'spec_link1': 'specA'})
            ]}},
        }
    }


@pytest.mark.parametrize("testdriver,expected_extra", [
    (True, {"testdriver": True}),
    # Don't bloat the manifest with the `testdriver=False` default.
    (False, {}),
])
def test_dump_testdriver(testdriver, expected_extra):
    m = manifest.Manifest("")
    source_file = SourceFileWithTest("a" + os.path.sep + "b", "0"*40, item.RefTest,
                                     testdriver=testdriver)

    tree, sourcefile_mock = tree_and_sourcefile_mocks([(source_file, None, True)])
    with mock.patch("tools.manifest.manifest.SourceFile", side_effect=sourcefile_mock):
        assert m.update(tree) is True

    assert m.to_json() == {
        'version': 9,
        'url_base': '/',
        'items': {
            'reftest': {'a': {'b': [
                '0000000000000000000000000000000000000000',
                (mock.ANY, [], expected_extra)
            ]}},
        }
    }
