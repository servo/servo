# mypy: ignore-errors

import os
import sys
import tempfile

import pytest

from mozlog import structured
from ..testloader import TestFilter, TestLoader, TagFilter
from ..testloader import read_include_from_file
from .test_wpttest import make_mock_manifest

here = os.path.dirname(__file__)
sys.path.insert(0, os.path.join(here, os.pardir, os.pardir, os.pardir))
from manifest.manifest import Manifest as WPTManifest

structured.set_default_logger(structured.structuredlog.StructuredLogger("TestLoader"))

include_ini = """\
skip: true
[test_\u53F0]
  skip: false
"""


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
