import os

import mock

import hypothesis as h
import hypothesis.strategies as hs

from six import iteritems

from .. import manifest, sourcefile, item, utils

MYPY = False
if MYPY:
    # MYPY is set to True when run under Mypy.
    from typing import Any
    from typing import Type


def SourceFileWithTest(path, hash, cls, **kwargs):
    # type: (str, str, Type[item.ManifestItem], **Any) -> sourcefile.SourceFile
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

def SourceFileWithTests(path, hash, cls, variants):
    # type: (str, str, Type[item.URLManifestItem], **Any) -> sourcefile.SourceFile
    rel_path_parts = tuple(path.split(os.path.sep))
    s = mock.Mock(rel_path=path,
                  rel_path_parts=rel_path_parts,
                  hash=hash)
    tests = [cls("/foobar", path, "/", item[0], **item[1]) for item in variants]
    s.manifest_items = mock.Mock(return_value=(cls.item_type, tests))
    return s  # type: ignore


@hs.composite
def sourcefile_strategy(draw):
    item_classes = [item.TestharnessTest, item.RefTest,
                    item.ManualTest, item.WebDriverSpecTest,
                    item.ConformanceCheckerTest, item.SupportFile]
    cls = draw(hs.sampled_from(item_classes))

    path = u"a"
    rel_path_parts = tuple(path.split(os.path.sep))
    hash = draw(hs.text(alphabet=u"0123456789abcdef", min_size=40, max_size=40))
    s = mock.Mock(rel_path=path,
                  rel_path_parts=rel_path_parts,
                  hash=hash)

    if cls is item.RefTest:
        ref_path = u"b"
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
    names = hs.text(alphabet=hs.characters(blacklist_characters=u"\0/\\:*\"?<>|"), min_size=1)
    tree = hs.recursive(sourcefile_strategy(),
                        lambda children: hs.dictionaries(names, children, min_size=1),
                        max_leaves=10)

    generated_root = draw(tree)
    h.assume(isinstance(generated_root, dict))

    reftest_urls = []
    output = []
    stack = [((k,), v) for k, v in iteritems(generated_root)]
    while stack:
        path, node = stack.pop()
        if isinstance(node, dict):
            stack.extend((path + (k,), v) for k, v in iteritems(node))
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
                    reference = hs.tuples(hs.sampled_from([u"==", u"!="]),
                                          possible_urls)
                    references = hs.lists(reference, min_size=1, unique=True)
                    test_item.references = draw(references)
                    reftest_urls.append(test_item.url)
            output.append(node)

    return output


@h.given(manifest_tree())
# FIXME: Workaround for https://github.com/web-platform-tests/wpt/issues/22758
@h.settings(suppress_health_check=(h.HealthCheck.too_slow,))
@h.example([SourceFileWithTest("a", "0"*40, item.ConformanceCheckerTest)])
def test_manifest_to_json(s):
    m = manifest.Manifest()

    assert m.update((item, True) for item in s) is True

    json_str = m.to_json()
    loaded = manifest.Manifest.from_json("/", json_str)

    assert list(loaded) == list(m)

    assert loaded.to_json() == json_str


@h.given(manifest_tree())
# FIXME: Workaround for https://github.com/web-platform-tests/wpt/issues/22758
@h.settings(suppress_health_check=(h.HealthCheck.too_slow,))
@h.example([SourceFileWithTest("a", "0"*40, item.TestharnessTest)])
@h.example([SourceFileWithTest("a", "0"*40, item.RefTest, references=[("/aa", "==")])])
def test_manifest_idempotent(s):
    m = manifest.Manifest()

    assert m.update((item, True) for item in s) is True

    m1 = list(m)

    assert m.update((item, True) for item in s) is False

    assert list(m) == m1


def test_manifest_to_json_forwardslash():
    m = manifest.Manifest()

    s = SourceFileWithTest("a" + os.path.sep + "b", "0"*40, item.TestharnessTest)

    assert m.update([(s, True)]) is True

    assert m.to_json() == {
        'version': 8,
        'url_base': '/',
        'items': {
            'testharness': {'a': {'b': [
                '0000000000000000000000000000000000000000',
                (None, {})
            ]}},
        }
    }



def test_reftest_computation_chain():
    m = manifest.Manifest()

    s1 = SourceFileWithTest("test1", "0"*40, item.RefTest, references=[("/test2", "==")])
    s2 = SourceFileWithTest("test2", "0"*40, item.RefTest, references=[("/test3", "==")])

    m.update([(s1, True), (s2, True)])

    test1 = s1.manifest_items()[1][0]
    test2 = s2.manifest_items()[1][0]

    assert list(m) == [("reftest", test1.path, {test1}),
                       ("reftest", test2.path, {test2})]


def test_iterpath():
    m = manifest.Manifest()

    sources = [SourceFileWithTest("test1", "0"*40, item.RefTest, references=[("/test1-ref", "==")]),
               SourceFileWithTests("test2", "1"*40, item.TestharnessTest, [("test2-1.html", {}),
                                                                           ("test2-2.html", {})]),
               SourceFileWithTest("test3", "0"*40, item.TestharnessTest)]
    m.update([(s, True) for s in sources])

    assert {item.url for item in m.iterpath("test2")} == {"/test2-1.html",
                                                          "/test2-2.html"}
    assert set(m.iterpath("missing")) == set()


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

    m.update([(s1.rel_path, False)])

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
        'items': {'testharness': {'test1': [
            "1"*40,
            (None, {'timeout': 'long'})
        ]}},
        'url_base': '/',
        'version': 8
    }
