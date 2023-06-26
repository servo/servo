# mypy: ignore-errors

import os
import sys
import tempfile

import pytest

from mozlog import structured
from ..testloader import (
    DirectoryHashChunker,
    IDHashChunker,
    PathHashChunker,
    TestFilter,
    TestLoader,
    TagFilter,
    read_include_from_file,
)
from .test_wpttest import make_mock_manifest

here = os.path.dirname(__file__)
sys.path.insert(0, os.path.join(here, os.pardir, os.pardir, os.pardir))
from manifest.manifest import Manifest as WPTManifest

structured.set_default_logger(structured.structuredlog.StructuredLogger("TestLoader"))

TestFilter.__test__ = False
TestLoader.__test__ = False

include_ini = """\
skip: true
[test_\u53F0]
  skip: false
"""


@pytest.fixture
def manifest():
    manifest_json = {
        "items": {
            "testharness": {
                "a": {
                    "foo.html": [
                        "abcdef123456",
                        ["a/foo.html?b", {}],
                        ["a/foo.html?c", {}],
                    ],
                    "bar.html": [
                        "uvwxyz987654",
                        [None, {}],
                    ],
                }
            }
        },
        "url_base": "/",
        "version": 8,
    }
    return WPTManifest.from_json("/", manifest_json)



def test_loader_h2_tests():
    manifest_json = {
        "items": {
            "testharness": {
                "a": {
                    "foo.html": [
                        "abcdef123456",
                        [None, {}],
                    ],
                    "bar.h2.html": [
                        "uvwxyz987654",
                        [None, {}],
                    ],
                }
            }
        },
        "url_base": "/",
        "version": 8,
    }
    manifest = WPTManifest.from_json("/", manifest_json)

    # By default, the loader should include the h2 test.
    loader = TestLoader({manifest: {"metadata_path": ""}}, ["testharness"], None)
    assert "testharness" in loader.tests
    assert len(loader.tests["testharness"]) == 2
    assert len(loader.disabled_tests) == 0

    # We can also instruct it to skip them.
    loader = TestLoader({manifest: {"metadata_path": ""}}, ["testharness"], None, include_h2=False)
    assert "testharness" in loader.tests
    assert len(loader.tests["testharness"]) == 1
    assert "testharness" in loader.disabled_tests
    assert len(loader.disabled_tests["testharness"]) == 1
    assert loader.disabled_tests["testharness"][0].url == "/a/bar.h2.html"


@pytest.mark.xfail(sys.platform == "win32",
                   reason="NamedTemporaryFile cannot be reopened on Win32")
def test_include_file():
    test_cases = """
# This is a comment
/foo/bar-error.https.html
/foo/bar-success.https.html
/foo/idlharness.https.any.html
/foo/idlharness.https.any.worker.html
    """

    with tempfile.NamedTemporaryFile(mode="wt") as f:
        f.write(test_cases)
        f.flush()

        include = read_include_from_file(f.name)

        assert len(include) == 4
        assert "/foo/bar-error.https.html" in include
        assert "/foo/bar-success.https.html" in include
        assert "/foo/idlharness.https.any.html" in include
        assert "/foo/idlharness.https.any.worker.html" in include


@pytest.mark.xfail(sys.platform == "win32",
                   reason="NamedTemporaryFile cannot be reopened on Win32")
def test_filter_unicode():
    tests = make_mock_manifest(("test", "a", 10), ("test", "a/b", 10),
                               ("test", "c", 10))

    with tempfile.NamedTemporaryFile("wb", suffix=".ini") as f:
        f.write(include_ini.encode('utf-8'))
        f.flush()

        TestFilter(manifest_path=f.name, test_manifests=tests)


def test_loader_filter_tags():
    manifest_json = {
        "items": {
            "testharness": {
                "a": {
                    "foo.html": [
                        "abcdef123456",
                        [None, {}],
                    ],
                    "bar.html": [
                        "uvwxyz987654",
                        [None, {}],
                    ],
                }
            }
        },
        "url_base": "/",
        "version": 8,
    }
    manifest = WPTManifest.from_json("/", manifest_json)

    tmpdir_kwargs = {}
    if sys.version_info.major >= 3 and sys.version_info.minor >= 10:
        tmpdir_kwargs["ignore_cleanup_errors"] = True
    with tempfile.TemporaryDirectory(**tmpdir_kwargs) as metadata_path:
        a_path = os.path.join(metadata_path, "a")
        os.makedirs(a_path)
        with open(os.path.join(a_path, "bar.html.ini"), "w") as f:
            f.write("tags: [test-include]\n")

        loader = TestLoader({manifest: {"metadata_path": metadata_path}}, ["testharness"], None)
        assert len(loader.tests["testharness"]) == 2

        loader = TestLoader({manifest: {"metadata_path": metadata_path}}, ["testharness"], None,
                            test_filters=[TagFilter({"test-include"})])
        assert len(loader.tests["testharness"]) == 1
        assert loader.tests["testharness"][0].id == "/a/bar.html"
        assert loader.tests["testharness"][0].tags == {"dir:a", "test-include"}


def test_chunk_hash(manifest):
    chunker1 = PathHashChunker(total_chunks=2, chunk_number=1)
    chunker2 = PathHashChunker(total_chunks=2, chunk_number=2)
    # Check that the chunkers partition the manifest (i.e., each item is
    # assigned to exactly one chunk).
    items = sorted([*chunker1(manifest), *chunker2(manifest)],
                   key=lambda item: item[1])
    assert len(items) == 2
    test_type, test_path, tests = items[0]
    assert test_type == "testharness"
    assert test_path == os.path.join("a", "bar.html")
    assert {test.id for test in tests} == {"/a/bar.html"}
    test_type, test_path, tests = items[1]
    assert test_type == "testharness"
    assert test_path == os.path.join("a", "foo.html")
    assert {test.id for test in tests} == {"/a/foo.html?b", "/a/foo.html?c"}


def test_chunk_id_hash(manifest):
    chunker1 = IDHashChunker(total_chunks=2, chunk_number=1)
    chunker2 = IDHashChunker(total_chunks=2, chunk_number=2)
    items = []
    for test_type, test_path, tests in [*chunker1(manifest), *chunker2(manifest)]:
        assert len(tests) > 0
        items.extend((test_type, test_path, test) for test in tests)
    assert len(items) == 3
    items.sort(key=lambda item: item[2].id)
    test_type, test_path, test = items[0]
    assert test_type == "testharness"
    assert test_path == os.path.join("a", "bar.html")
    assert test.id == "/a/bar.html"
    test_type, test_path, test = items[1]
    assert test_type == "testharness"
    assert test_path == os.path.join("a", "foo.html")
    assert test.id == "/a/foo.html?b"
    test_type, test_path, test = items[2]
    assert test_type == "testharness"
    assert test_path == os.path.join("a", "foo.html")
    assert test.id == "/a/foo.html?c"


def test_chunk_dir_hash(manifest):
    chunker1 = DirectoryHashChunker(total_chunks=2, chunk_number=1)
    chunker2 = DirectoryHashChunker(total_chunks=2, chunk_number=2)
    # Check that tests in the same directory are located in the same chunk
    # (which particular chunk is irrelevant).
    empty_chunk, chunk_a = sorted([
        list(chunker1(manifest)),
        list(chunker2(manifest)),
    ], key=len)
    assert len(empty_chunk) == 0
    assert len(chunk_a) == 2
    test_type, test_path, tests = chunk_a[0]
    assert test_type == "testharness"
    assert test_path == os.path.join("a", "bar.html")
    assert {test.id for test in tests} == {"/a/bar.html"}
    test_type, test_path, tests = chunk_a[1]
    assert test_type == "testharness"
    assert test_path == os.path.join("a", "foo.html")
    assert {test.id for test in tests} == {"/a/foo.html?b", "/a/foo.html?c"}
