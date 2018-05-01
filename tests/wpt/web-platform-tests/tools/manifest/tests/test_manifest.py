import platform
import os

import mock

import hypothesis as h
import hypothesis.strategies as hs

import pytest

from .. import manifest, item, sourcefile, utils


def SourceFileWithTest(path, hash, cls, *args):
    s = mock.Mock(rel_path=path, hash=hash)
    test = cls(s, utils.rel_path_to_url(path), *args)
    s.manifest_items = mock.Mock(return_value=(cls.item_type, [test]))
    return s

def SourceFileWithTests(path, hash, cls, variants):
    s = mock.Mock(rel_path=path, hash=hash)
    tests = [cls(s, item[0], *item[1:]) for item in variants]
    s.manifest_items = mock.Mock(return_value=(cls.item_type, tests))
    return s


@hs.composite
def rel_dir_file_path(draw):
    length = draw(hs.integers(min_value=1, max_value=20))
    if length == 1:
        return "a"
    else:
        remaining = length - 2
        if os.path.sep == "/":
            alphabet = "a/"
        elif os.path.sep == "\\":
            alphabet = "a/\\"
        else:
            assert False, "uhhhh, this platform is weird"
        mid = draw(hs.text(alphabet=alphabet, min_size=remaining, max_size=remaining))
        return os.path.normcase("a" + mid + "a")


@hs.composite
def sourcefile_strategy(draw):
    item_classes = [item.TestharnessTest, item.RefTest, item.RefTestNode,
                    item.ManualTest, item.Stub, item.WebdriverSpecTest,
                    item.ConformanceCheckerTest, item.SupportFile]
    cls = draw(hs.sampled_from(item_classes))

    path = draw(rel_dir_file_path())
    hash = draw(hs.text(alphabet="0123456789abcdef", min_size=40, max_size=40))
    s = mock.Mock(rel_path=path, hash=hash)

    if cls in (item.RefTest, item.RefTestNode):
        ref_path = draw(rel_dir_file_path())
        h.assume(path != ref_path)
        ref_eq = draw(hs.sampled_from(["==", "!="]))
        test = cls(s, utils.rel_path_to_url(path), [(utils.rel_path_to_url(ref_path), ref_eq)])
    elif cls is item.SupportFile:
        test = cls(s)
    else:
        test = cls(s, utils.rel_path_to_url(path))

    s.manifest_items = mock.Mock(return_value=(cls.item_type, [test]))
    return s


@h.given(hs.lists(sourcefile_strategy(),
                  min_size=1, average_size=10, max_size=1000,
                  unique_by=lambda x: x.rel_path))
@h.example([SourceFileWithTest("a", "0"*40, item.ConformanceCheckerTest)])
def test_manifest_to_json(s):
    m = manifest.Manifest()

    assert m.update(s) is True

    json_str = m.to_json()
    loaded = manifest.Manifest.from_json("/", json_str)

    assert list(loaded) == list(m)

    assert loaded.to_json() == json_str


@h.given(hs.lists(sourcefile_strategy(),
                  min_size=1, average_size=10,
                  unique_by=lambda x: x.rel_path))
@h.example([SourceFileWithTest("a", "0"*40, item.TestharnessTest)])
@h.example([SourceFileWithTest("a", "0"*40, item.RefTest, [("/aa", "==")])])
def test_manifest_idempotent(s):
    m = manifest.Manifest()

    assert m.update(s) is True

    m1 = list(m)

    assert m.update(s) is False

    assert list(m) == m1


def test_manifest_to_json_forwardslash():
    m = manifest.Manifest()

    s = SourceFileWithTest("a/b", "0"*40, item.TestharnessTest)

    assert m.update([s]) is True

    assert m.to_json() == {
        'paths': {
            'a/b': ('0000000000000000000000000000000000000000', 'testharness')
        },
        'version': 4,
        'url_base': '/',
        'items': {
            'reftest': {},
            'reftest_node': {},
            'testharness': {
                'a/b': [['/a/b', {}]]
            }
        }
    }


def test_manifest_to_json_backslash():
    m = manifest.Manifest()

    s = SourceFileWithTest("a\\b", "0"*40, item.TestharnessTest)

    if os.path.sep == "\\":
        assert m.update([s]) is True

        assert m.to_json() == {
            'paths': {
                'a/b': ('0000000000000000000000000000000000000000', 'testharness')
            },
            'version': 4,
            'url_base': '/',
            'items': {
                'reftest': {},
                'reftest_node': {},
                'testharness': {
                    'a/b': [['/a/b', {}]]
                }
            }
        }
    else:
        with pytest.raises(ValueError):
            # one of these must raise ValueError
            # the first must return True if it doesn't raise
            assert m.update([s]) is True
            m.to_json()


def test_manifest_from_json_backslash():
    json_obj = {
        'paths': {
            'a\\b': ('0000000000000000000000000000000000000000', 'testharness')
        },
        'version': 4,
        'url_base': '/',
        'items': {
            'reftest': {},
            'reftest_node': {},
            'testharness': {
                'a\\b': [['/a/b', {}]]
            }
        }
    }

    with pytest.raises(ValueError):
        manifest.Manifest.from_json("/", json_obj)


def test_reftest_computation_chain():
    m = manifest.Manifest()

    s1 = SourceFileWithTest("test1", "0"*40, item.RefTest, [("/test2", "==")])
    s2 = SourceFileWithTest("test2", "0"*40, item.RefTest, [("/test3", "==")])

    m.update([s1, s2])

    test1 = s1.manifest_items()[1][0]
    test2 = s2.manifest_items()[1][0]
    test2_node = test2.to_RefTestNode()

    assert list(m) == [("reftest", test1.path, {test1}),
                       ("reftest_node", test2.path, {test2_node})]


def test_reftest_computation_chain_update_add():
    m = manifest.Manifest()

    s2 = SourceFileWithTest("test2", "0"*40, item.RefTest, [("/test3", "==")])
    test2 = s2.manifest_items()[1][0]

    assert m.update([s2]) is True

    assert list(m) == [("reftest", test2.path, {test2})]

    s1 = SourceFileWithTest("test1", "0"*40, item.RefTest, [("/test2", "==")])
    test1 = s1.manifest_items()[1][0]

    # s2's hash is unchanged, but it has gone from a test to a node
    assert m.update([s1, s2]) is True

    test2_node = test2.to_RefTestNode()

    assert list(m) == [("reftest", test1.path, {test1}),
                       ("reftest_node", test2.path, {test2_node})]


def test_reftest_computation_chain_update_remove():
    m = manifest.Manifest()

    s1 = SourceFileWithTest("test1", "0"*40, item.RefTest, [("/test2", "==")])
    s2 = SourceFileWithTest("test2", "0"*40, item.RefTest, [("/test3", "==")])

    assert m.update([s1, s2]) is True

    test1 = s1.manifest_items()[1][0]
    test2 = s2.manifest_items()[1][0]
    test2_node = test2.to_RefTestNode()

    assert list(m) == [("reftest", test1.path, {test1}),
                       ("reftest_node", test2.path, {test2_node})]

    # s2's hash is unchanged, but it has gone from a node to a test
    assert m.update([s2]) is True

    assert list(m) == [("reftest", test2.path, {test2})]


def test_reftest_computation_chain_update_test_type():
    m = manifest.Manifest()

    s1 = SourceFileWithTest("test", "0"*40, item.RefTest, [("/test-ref", "==")])

    assert m.update([s1]) is True

    test1 = s1.manifest_items()[1][0]

    assert list(m) == [("reftest", test1.path, {test1})]

    # test becomes a testharness test (hash change because that is determined
    # based on the file contents). The updated manifest should not includes the
    # old reftest.
    s2 = SourceFileWithTest("test", "1"*40, item.TestharnessTest)
    assert m.update([s2]) is True

    test2 = s2.manifest_items()[1][0]

    assert list(m) == [("testharness", test2.path, {test2})]


def test_reftest_computation_chain_update_node_change():
    m = manifest.Manifest()

    s1 = SourceFileWithTest("test1", "0"*40, item.RefTest, [("/test2", "==")])
    s2 = SourceFileWithTest("test2", "0"*40, item.RefTestNode, [("/test3", "==")])

    assert m.update([s1, s2]) is True

    test1 = s1.manifest_items()[1][0]
    test2 = s2.manifest_items()[1][0]

    assert list(m) == [("reftest", test1.path, {test1}),
                       ("reftest_node", test2.path, {test2})]

    #test2 changes to support type
    s2 = SourceFileWithTest("test2", "1"*40, item.SupportFile)

    assert m.update([s1,s2]) is True
    test3 = s2.manifest_items()[1][0]

    assert list(m) == [("reftest", test1.path, {test1}),
                       ("support", test3.path, {test3})]


def test_iterpath():
    m = manifest.Manifest()

    sources = [SourceFileWithTest("test1", "0"*40, item.RefTest, [("/test1-ref", "==")]),
               SourceFileWithTest("test2", "0"*40, item.RefTest, [("/test2-ref", "==")]),
               SourceFileWithTests("test2", "0"*40, item.TestharnessTest, [("/test2-1.html",),
                                                                           ("/test2-2.html",)]),
               SourceFileWithTest("test3", "0"*40, item.TestharnessTest)]
    m.update(sources)

    assert set(item.url for item in m.iterpath("test2")) == set(["/test2",
                                                                 "/test2-1.html",
                                                                 "/test2-2.html"])
    assert set(m.iterpath("missing")) == set()


def test_reftest_node_by_url():
    m = manifest.Manifest()

    s1 = SourceFileWithTest("test1", "0"*40, item.RefTest, [("/test2", "==")])
    s2 = SourceFileWithTest("test2", "0"*40, item.RefTest, [("/test3", "==")])

    m.update([s1, s2])

    test1 = s1.manifest_items()[1][0]
    test2 = s2.manifest_items()[1][0]
    test2_node = test2.to_RefTestNode()

    assert m.reftest_nodes_by_url == {"/test1": test1,
                                      "/test2": test2_node}
    m._reftest_nodes_by_url = None
    assert m.reftest_nodes_by_url == {"/test1": test1,
                                      "/test2": test2_node}
