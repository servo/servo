# mypy: allow-untyped-defs

import os

import pytest

from io import BytesIO
from ...lint.lint import check_global_metadata
from ..sourcefile import SourceFile, read_script_metadata, js_meta_re, python_meta_re


def create(filename, contents=b""):
    assert isinstance(contents, bytes)
    return SourceFile("/", filename, "/", contents=contents)


def items(s):
    item_type, items = s.manifest_items()
    if item_type == "support":
        return []
    else:
        return [(item_type, item.url) for item in items]


@pytest.mark.parametrize("rel_path", [
    ".gitignore",
    ".travis.yml",
    "MANIFEST.json",
    "tools/test.html",
    "resources/test.html",
    "common/test.html",
    "support/test.html",
    "css21/archive/test.html",
    "conformance-checkers/test.html",
    "conformance-checkers/README.md",
    "conformance-checkers/html/Makefile",
    "conformance-checkers/html/test.html",
    "foo/tools/test.html",
    "foo/resources/test.html",
    "foo/support/test.html",
    "foo/foo-manual.html.headers",
    "crashtests/foo.html.ini",
    "css/common/test.html",
    "css/CSS2/archive/test.html",
])
def test_name_is_non_test(rel_path):
    s = create(rel_path)
    assert s.name_is_non_test or s.name_is_conformance_support

    assert not s.content_is_testharness

    assert items(s) == []


@pytest.mark.parametrize("rel_path", [
    "foo/common/test.html",
    "foo/conformance-checkers/test.html",
    "foo/_certs/test.html",
    "foo/css21/archive/test.html",
    "foo/CSS2/archive/test.html",
    "css/css21/archive/test.html",
    "foo/test-support.html",
])
def test_not_name_is_non_test(rel_path):
    s = create(rel_path)
    assert not (s.name_is_non_test or s.name_is_conformance_support)
    # We aren't actually asserting what type of test these are, just their
    # name doesn't prohibit them from being tests.


@pytest.mark.parametrize("rel_path", [
    "foo/foo-manual.html",
    "html/test-manual.html",
    "html/test-manual.xhtml",
    "html/test-manual.https.html",
    "html/test-manual.https.xhtml"
])
def test_name_is_manual(rel_path):
    s = create(rel_path)
    assert not s.name_is_non_test
    assert s.name_is_manual

    assert not s.content_is_testharness

    assert items(s) == [("manual", "/" + rel_path)]


@pytest.mark.parametrize("rel_path", [
    "html/test-visual.html",
    "html/test-visual.xhtml",
])
def test_name_is_visual(rel_path):
    s = create(rel_path)
    assert not s.name_is_non_test
    assert s.name_is_visual

    assert not s.content_is_testharness

    assert items(s) == [("visual", "/" + rel_path)]


@pytest.mark.parametrize("rel_path", [
    "css-namespaces-3/reftest/ref-lime-1.xml",
    "css21/reference/pass_if_box_ahem.html",
    "css21/csswg-issues/submitted/css2.1/reference/ref-green-box-100x100.xht",
    "selectors-3/selectors-empty-001-ref.xml",
    "css21/text/text-indent-wrap-001-notref-block-margin.xht",
    "css21/text/text-indent-wrap-001-notref-block-margin.xht",
    "css21/css-e-notation-ref-1.html",
    "css21/floats/floats-placement-vertical-004-ref2.xht",
    "css21/box/rtl-linebreak-notref1.xht",
    "css21/box/rtl-linebreak-notref2.xht",
    "html/canvas/element/drawing-images-to-the-canvas/drawimage_html_image_5_ref.html",
    "html/canvas/element/line-styles/lineto_ref.html",
    "html/rendering/non-replaced-elements/the-fieldset-element-0/ref.html"
])
def test_name_is_reference(rel_path):
    s = create(rel_path)
    assert not s.name_is_non_test
    assert s.name_is_reference

    assert not s.content_is_testharness

    assert items(s) == []


def test_name_is_tentative():
    s = create("css/css-ui/appearance-revert-001.tentative.html")
    assert s.name_is_tentative

    s = create("css/css-ui/tentative/appearance-revert-001.html")
    assert s.name_is_tentative

    s = create("css/css-ui/appearance-revert-001.html")
    assert not s.name_is_tentative


@pytest.mark.parametrize("rel_path", [
    "webdriver/tests/foo.py",
    "webdriver/tests/print/foo.py",
    "webdriver/tests/foo-crash.py",
    "webdriver/tests/foo-visual.py",
])
def test_name_is_webdriver(rel_path):
    s = create(rel_path)
    assert s.name_is_webdriver

    item_type, items = s.manifest_items()
    assert item_type == "wdspec"


def test_worker():
    s = create("html/test.worker.js")
    assert not s.name_is_non_test
    assert not s.name_is_manual
    assert not s.name_is_visual
    assert not s.name_is_multi_global
    assert s.name_is_worker
    assert not s.name_is_window
    assert not s.name_is_reference

    assert not s.content_is_testharness

    item_type, items = s.manifest_items()
    assert item_type == "testharness"

    expected_urls = [
        "/html/test.worker.html",
    ]
    assert len(items) == len(expected_urls)

    for item, url in zip(items, expected_urls):
        assert item.url == url
        assert item.timeout is None


def test_window():
    s = create("html/test.window.js")
    assert not s.name_is_non_test
    assert not s.name_is_manual
    assert not s.name_is_visual
    assert not s.name_is_multi_global
    assert not s.name_is_worker
    assert s.name_is_window
    assert not s.name_is_reference

    assert not s.content_is_testharness

    item_type, items = s.manifest_items()
    assert item_type == "testharness"

    expected_urls = [
        "/html/test.window.html",
    ]
    assert len(items) == len(expected_urls)

    for item, url in zip(items, expected_urls):
        assert item.url == url
        assert item.timeout is None


def test_worker_long_timeout():
    contents = b"""// META: timeout=long
importScripts('/resources/testharness.js')
test()"""

    metadata = list(read_script_metadata(BytesIO(contents), js_meta_re))
    assert metadata == [("timeout", "long")]

    s = create("html/test.worker.js", contents=contents)
    assert s.name_is_worker

    item_type, items = s.manifest_items()
    assert item_type == "testharness"

    for item in items:
        assert item.timeout == "long"


def test_window_long_timeout():
    contents = b"""// META: timeout=long
test()"""

    metadata = list(read_script_metadata(BytesIO(contents), js_meta_re))
    assert metadata == [("timeout", "long")]

    s = create("html/test.window.js", contents=contents)
    assert s.name_is_window

    item_type, items = s.manifest_items()
    assert item_type == "testharness"

    for item in items:
        assert item.timeout == "long"


def test_worker_with_variants():
    contents = b"""// META: variant=?default
// META: variant=?wss
test()"""

    s = create("html/test.worker.js", contents=contents)
    assert not s.name_is_non_test
    assert not s.name_is_manual
    assert not s.name_is_visual
    assert not s.name_is_multi_global
    assert s.name_is_worker
    assert not s.name_is_window
    assert not s.name_is_reference

    assert not s.content_is_testharness

    item_type, items = s.manifest_items()
    assert item_type == "testharness"

    expected_urls = [
        "/html/test.worker.html" + suffix
        for suffix in ["?default", "?wss"]
    ]
    assert len(items) == len(expected_urls)

    for item, url in zip(items, expected_urls):
        assert item.url == url
        assert item.timeout is None


def test_window_with_variants():
    contents = b"""// META: variant=?default
// META: variant=?wss
test()"""

    s = create("html/test.window.js", contents=contents)
    assert not s.name_is_non_test
    assert not s.name_is_manual
    assert not s.name_is_visual
    assert not s.name_is_multi_global
    assert not s.name_is_worker
    assert s.name_is_window
    assert not s.name_is_reference

    assert not s.content_is_testharness

    item_type, items = s.manifest_items()
    assert item_type == "testharness"

    expected_urls = [
        "/html/test.window.html" + suffix
        for suffix in ["?default", "?wss"]
    ]
    assert len(items) == len(expected_urls)

    for item, url in zip(items, expected_urls):
        assert item.url == url
        assert item.timeout is None


def test_python_long_timeout():
    contents = b"""# META: timeout=long

"""

    metadata = list(read_script_metadata(BytesIO(contents),
                                         python_meta_re))
    assert metadata == [("timeout", "long")]

    s = create("webdriver/test.py", contents=contents)
    assert s.name_is_webdriver

    item_type, items = s.manifest_items()
    assert item_type == "wdspec"

    for item in items:
        assert item.timeout == "long"


def test_multi_global():
    s = create("html/test.any.js")
    assert not s.name_is_non_test
    assert not s.name_is_manual
    assert not s.name_is_visual
    assert s.name_is_multi_global
    assert not s.name_is_worker
    assert not s.name_is_reference

    assert not s.content_is_testharness

    item_type, items = s.manifest_items()
    assert item_type == "testharness"

    expected_urls = [
        "/html/test.any.html",
        "/html/test.any.worker.html",
    ]
    assert len(items) == len(expected_urls)

    for item, url in zip(items, expected_urls):
        assert item.url == url
        assert item.timeout is None


def test_multi_global_long_timeout():
    contents = b"""// META: timeout=long
importScripts('/resources/testharness.js')
test()"""

    metadata = list(read_script_metadata(BytesIO(contents), js_meta_re))
    assert metadata == [("timeout", "long")]

    s = create("html/test.any.js", contents=contents)
    assert s.name_is_multi_global

    item_type, items = s.manifest_items()
    assert item_type == "testharness"

    for item in items:
        assert item.timeout == "long"


@pytest.mark.parametrize("input,expected", [
    (b"window", {"window"}),
    (b"sharedworker", {"sharedworker"}),
    (b"sharedworker,serviceworker", {"serviceworker", "sharedworker"}),
    (b"worker", {"dedicatedworker", "serviceworker", "sharedworker"}),
])
def test_multi_global_with_custom_globals(input, expected):
    contents = b"""// META: global=%s
test()""" % input

    assert list(check_global_metadata(input)) == []

    s = create("html/test.any.js", contents=contents)
    assert not s.name_is_non_test
    assert not s.name_is_manual
    assert not s.name_is_visual
    assert s.name_is_multi_global
    assert not s.name_is_worker
    assert not s.name_is_reference

    assert not s.content_is_testharness

    item_type, items = s.manifest_items()
    assert item_type == "testharness"

    urls = {
        "dedicatedworker": "/html/test.any.worker.html",
        "serviceworker": "/html/test.any.serviceworker.html",
        "sharedworker": "/html/test.any.sharedworker.html",
        "window": "/html/test.any.html",
    }

    expected_urls = sorted(urls[ty] for ty in expected)
    assert len(items) == len(expected_urls)

    for item, url in zip(items, expected_urls):
        assert item.url == url
        assert item.jsshell is False
        assert item.timeout is None


def test_multi_global_with_jsshell_globals():
    contents = b"""// META: global=window,dedicatedworker,jsshell
test()"""

    s = create("html/test.any.js", contents=contents)
    assert not s.name_is_non_test
    assert not s.name_is_manual
    assert not s.name_is_visual
    assert s.name_is_multi_global
    assert not s.name_is_worker
    assert not s.name_is_reference

    assert not s.content_is_testharness

    item_type, items = s.manifest_items()
    assert item_type == "testharness"

    expected = [
        ("/html/test.any.html", False),
        ("/html/test.any.js", True),
        ("/html/test.any.worker.html", False),
    ]
    assert len(items) == len(expected)

    for item, (url, jsshell) in zip(items, expected):
        assert item.url == url
        assert item.jsshell == jsshell
        assert item.timeout is None


def test_multi_global_with_variants():
    contents = b"""// META: global=window,worker
// META: variant=?default
// META: variant=?wss
test()"""

    s = create("html/test.any.js", contents=contents)
    assert not s.name_is_non_test
    assert not s.name_is_manual
    assert not s.name_is_visual
    assert s.name_is_multi_global
    assert not s.name_is_worker
    assert not s.name_is_reference

    assert not s.content_is_testharness

    item_type, items = s.manifest_items()
    assert item_type == "testharness"

    urls = {
        "dedicatedworker": "/html/test.any.worker.html",
        "serviceworker": "/html/test.any.serviceworker.html",
        "sharedworker": "/html/test.any.sharedworker.html",
        "window": "/html/test.any.html",
    }

    expected_urls = sorted(
        urls[ty] + suffix
        for ty in ["dedicatedworker", "serviceworker", "sharedworker", "window"]
        for suffix in ["?default", "?wss"]
    )
    assert len(items) == len(expected_urls)

    for item, url in zip(items, expected_urls):
        assert item.url == url
        assert item.timeout is None


@pytest.mark.parametrize("input,expected", [
    (b"""//META: foo=bar\n""", [("foo", "bar")]),
    (b"""// META: foo=bar\n""", [("foo", "bar")]),
    (b"""//  META: foo=bar\n""", [("foo", "bar")]),
    (b"""\n// META: foo=bar\n""", []),
    (b""" // META: foo=bar\n""", []),
    (b"""// META: foo=bar\n// META: baz=quux\n""", [("foo", "bar"), ("baz", "quux")]),
    (b"""// META: foo=bar\n\n// META: baz=quux\n""", [("foo", "bar")]),
    (b"""// META: foo=bar\n// Start of the test\n// META: baz=quux\n""", [("foo", "bar")]),
    (b"""// META:\n""", []),
    (b"""// META: foobar\n""", []),
])
def test_script_metadata(input, expected):
    metadata = read_script_metadata(BytesIO(input), js_meta_re)
    assert list(metadata) == expected


@pytest.mark.parametrize("ext", ["htm", "html"])
def test_testharness(ext):
    content = b"<script src=/resources/testharness.js></script>"

    filename = "html/test." + ext
    s = create(filename, content)

    assert not s.name_is_non_test
    assert not s.name_is_manual
    assert not s.name_is_visual
    assert not s.name_is_multi_global
    assert not s.name_is_worker
    assert not s.name_is_reference

    assert s.content_is_testharness

    assert items(s) == [("testharness", "/" + filename)]


@pytest.mark.parametrize("variant", ["", "?foo", "#bar", "?foo#bar"])
def test_testharness_variant(variant):
    content = (b"<meta name=variant content=\"%s\">" % variant.encode("utf-8") +
               b"<meta name=variant content=\"?fixed\">" +
               b"<script src=/resources/testharness.js></script>")

    filename = "html/test.html"
    s = create(filename, content)

    s.test_variants = [variant, "?fixed"]


@pytest.mark.parametrize("variant", ["?", "#", "?#bar"])
def test_testharness_variant_invalid(variant):
    content = (b"<meta name=variant content=\"%s\">" % variant.encode("utf-8") +
               b"<meta name=variant content=\"?fixed\">" +
               b"<script src=/resources/testharness.js></script>")

    filename = "html/test.html"
    s = create(filename, content)

    with pytest.raises(ValueError):
        s.test_variants


def test_reftest_variant():
    content = (b"<meta name=variant content=\"?first\">" +
               b"<meta name=variant content=\"?second\">" +
               b"<link rel=\"match\" href=\"ref.html\">")

    s = create("html/test.html", contents=content)
    assert not s.name_is_non_test
    assert not s.name_is_manual
    assert not s.name_is_visual
    assert not s.name_is_worker
    assert not s.name_is_reference

    item_type, items = s.manifest_items()
    assert item_type == "reftest"

    actual_tests = [
        {"url": item.url, "refs": item.references}
        for item in items
    ]

    expected_tests = [
        {
            "url": "/html/test.html?first",
            "refs": [("/html/ref.html?first", "==")],
        },
        {
            "url": "/html/test.html?second",
            "refs": [("/html/ref.html?second", "==")],
        },
    ]

    assert actual_tests == expected_tests


@pytest.mark.parametrize("ext", ["htm", "html"])
def test_relative_testharness(ext):
    content = b"<script src=../resources/testharness.js></script>"

    filename = "html/test." + ext
    s = create(filename, content)

    assert not s.name_is_non_test
    assert not s.name_is_manual
    assert not s.name_is_visual
    assert not s.name_is_multi_global
    assert not s.name_is_worker
    assert not s.name_is_reference

    assert not s.content_is_testharness

    assert items(s) == []


@pytest.mark.parametrize("ext", ["xhtml", "xht", "xml"])
def test_testharness_xhtml(ext):
    content = b"""
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
</head>
<body/>
</html>
"""

    filename = "html/test." + ext
    s = create(filename, content)

    assert not s.name_is_non_test
    assert not s.name_is_manual
    assert not s.name_is_visual
    assert not s.name_is_multi_global
    assert not s.name_is_worker
    assert not s.name_is_reference

    assert s.content_is_testharness

    assert items(s) == [("testharness", "/" + filename)]


@pytest.mark.parametrize("ext", ["xhtml", "xht", "xml"])
def test_relative_testharness_xhtml(ext):
    content = b"""
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
<script src="../resources/testharness.js"></script>
<script src="../resources/testharnessreport.js"></script>
</head>
<body/>
</html>
"""

    filename = "html/test." + ext
    s = create(filename, content)

    assert not s.name_is_non_test
    assert not s.name_is_manual
    assert not s.name_is_visual
    assert not s.name_is_multi_global
    assert not s.name_is_worker
    assert not s.name_is_reference

    assert not s.content_is_testharness

    assert items(s) == []


def test_testharness_svg():
    content = b"""\
<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg"
     xmlns:h="http://www.w3.org/1999/xhtml"
     version="1.1"
     width="100%" height="100%" viewBox="0 0 400 400">
<title>Null test</title>
<h:script src="/resources/testharness.js"/>
<h:script src="/resources/testharnessreport.js"/>
</svg>
"""

    filename = "html/test.svg"
    s = create(filename, content)

    assert not s.name_is_non_test
    assert not s.name_is_manual
    assert not s.name_is_visual
    assert not s.name_is_multi_global
    assert not s.name_is_worker
    assert not s.name_is_reference

    assert s.root is not None
    assert s.content_is_testharness

    assert items(s) == [("testharness", "/" + filename)]


def test_relative_testharness_svg():
    content = b"""\
<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg"
     xmlns:h="http://www.w3.org/1999/xhtml"
     version="1.1"
     width="100%" height="100%" viewBox="0 0 400 400">
<title>Null test</title>
<h:script src="../resources/testharness.js"/>
<h:script src="../resources/testharnessreport.js"/>
</svg>
"""

    filename = "html/test.svg"
    s = create(filename, content)

    assert not s.name_is_non_test
    assert not s.name_is_manual
    assert not s.name_is_visual
    assert not s.name_is_multi_global
    assert not s.name_is_worker
    assert not s.name_is_reference

    assert s.root is not None
    assert not s.content_is_testharness

    assert items(s) == []


@pytest.mark.parametrize("filename", ["test", "test.test"])
def test_testharness_ext(filename):
    content = b"<script src=/resources/testharness.js></script>"

    s = create("html/" + filename, content)

    assert not s.name_is_non_test
    assert not s.name_is_manual
    assert not s.name_is_visual
    assert not s.name_is_multi_global
    assert not s.name_is_worker
    assert not s.name_is_reference

    assert not s.root
    assert not s.content_is_testharness

    assert items(s) == []


@pytest.mark.parametrize("ext", ["htm", "html"])
def test_testdriver(ext):
    content = b"<script src=/resources/testdriver.js></script>"

    filename = "html/test." + ext
    s = create(filename, content)

    assert s.has_testdriver


@pytest.mark.parametrize("ext", ["htm", "html"])
def test_relative_testdriver(ext):
    content = b"<script src=../resources/testdriver.js></script>"

    filename = "html/test." + ext
    s = create(filename, content)

    assert not s.has_testdriver


@pytest.mark.parametrize("ext", ["htm", "html"])
def test_reftest(ext):
    content = b"<link rel=match href=ref.html>"

    filename = "foo/test." + ext
    s = create(filename, content)

    assert not s.name_is_non_test
    assert not s.name_is_manual
    assert not s.name_is_visual
    assert not s.name_is_multi_global
    assert not s.name_is_worker
    assert not s.name_is_reference
    assert not s.content_is_testharness

    assert s.content_is_ref_node

    assert items(s) == [("reftest", "/" + filename)]


@pytest.mark.parametrize("ext", ["xht", "html", "xhtml", "htm", "xml", "svg"])
def test_css_visual(ext):
    content = b"""
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
<link rel="help" href="http://www.w3.org/TR/CSS21/box.html#bidi-box-model"/>
</head>
<body></body>
</html>
"""

    filename = "html/test." + ext
    s = create(filename, content)

    assert not s.name_is_non_test
    assert not s.name_is_manual
    assert not s.name_is_visual
    assert not s.name_is_multi_global
    assert not s.name_is_worker
    assert not s.name_is_reference
    assert not s.content_is_testharness
    assert not s.content_is_ref_node

    assert s.content_is_css_visual

    assert items(s) == [("visual", "/" + filename)]


@pytest.mark.parametrize("ext", ["xht", "xhtml", "xml"])
def test_xhtml_with_entity(ext):
    content = b"""
<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Strict//EN"
   "http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd">
<html xmlns="http://www.w3.org/1999/xhtml">
&nbsp;
</html>
"""

    filename = "html/test." + ext
    s = create(filename, content)

    assert s.root is not None

    assert items(s) == []


def test_no_parse():
    s = create("foo/bar.xml", "\uFFFF".encode("utf-8"))

    assert not s.name_is_non_test
    assert not s.name_is_manual
    assert not s.name_is_visual
    assert not s.name_is_multi_global
    assert not s.name_is_worker
    assert not s.name_is_reference
    assert not s.content_is_testharness
    assert not s.content_is_ref_node
    assert not s.content_is_css_visual

    assert items(s) == []


@pytest.mark.parametrize("input,expected", [
    ("aA", "aA"),
    ("a/b", "a/b" if os.name != "nt" else "a\\b"),
    ("a\\b", "a\\b")
])
def test_relpath_normalized(input, expected):
    s = create(input, b"")
    assert s.rel_path == expected


@pytest.mark.parametrize("url", [b"ref.html",
                                 b"\x20ref.html",
                                 b"ref.html\x20",
                                 b"\x09\x0a\x0c\x0d\x20ref.html\x09\x0a\x0c\x0d\x20"])
def test_reftest_url_whitespace(url):
    content = b"<link rel=match href='%s'>" % url
    s = create("foo/test.html", content)
    assert s.references == [("/foo/ref.html", "==")]


@pytest.mark.parametrize("url", [b"http://example.com/",
                                 b"\x20http://example.com/",
                                 b"http://example.com/\x20",
                                 b"\x09\x0a\x0c\x0d\x20http://example.com/\x09\x0a\x0c\x0d\x20"])
def test_spec_links_whitespace(url):
    content = b"<link rel=help href='%s'>" % url
    s = create("foo/test.html", content)
    assert s.spec_links == {"http://example.com/"}


@pytest.mark.parametrize("input,expected", [
    (b"""<link rel="help" title="Intel" href="foo">\n""", ["foo"]),
    (b"""<link rel=help title="Intel" href="foo">\n""", ["foo"]),
    (b"""<link  rel=help  href="foo" >\n""", ["foo"]),
    (b"""<link rel="author" href="foo">\n""", []),
    (b"""<link href="foo">\n""", []),
    (b"""<link rel="help" href="foo">\n<link rel="help" href="bar">\n""", ["foo", "bar"]),
    (b"""<link rel="help" href="foo">\n<script>\n""", ["foo"]),
    (b"""random\n""", []),
])
def test_spec_links_complex(input, expected):
    s = create("foo/test.html", input)
    assert s.spec_links == set(expected)


def test_url_base():
    contents = b"""// META: global=window,worker
// META: variant=?default
// META: variant=?wss
test()"""

    s = SourceFile("/", "html/test.any.js", "/_fake_base/", contents=contents)
    item_type, items = s.manifest_items()

    assert item_type == "testharness"

    assert [item.url for item in items] == ['/_fake_base/html/test.any.html?default',
                                            '/_fake_base/html/test.any.html?wss',
                                            '/_fake_base/html/test.any.serviceworker.html?default',
                                            '/_fake_base/html/test.any.serviceworker.html?wss',
                                            '/_fake_base/html/test.any.sharedworker.html?default',
                                            '/_fake_base/html/test.any.sharedworker.html?wss',
                                            '/_fake_base/html/test.any.worker.html?default',
                                            '/_fake_base/html/test.any.worker.html?wss']

    assert items[0].url_base == "/_fake_base/"


@pytest.mark.parametrize("fuzzy, expected", [
    (b"ref.html:1;200", {("/foo/test.html", "/foo/ref.html", "=="): [[1, 1], [200, 200]]}),
    (b"ref.html:0-1;100-200", {("/foo/test.html", "/foo/ref.html", "=="): [[0, 1], [100, 200]]}),
    (b"0-1;100-200", {None: [[0,1], [100, 200]]}),
    (b"maxDifference=1;totalPixels=200", {None: [[1, 1], [200, 200]]}),
    (b"totalPixels=200;maxDifference=1", {None: [[1, 1], [200, 200]]}),
    (b"totalPixels=200;1", {None: [[1, 1], [200, 200]]}),
    (b"maxDifference=1;200", {None: [[1, 1], [200, 200]]}),])
def test_reftest_fuzzy(fuzzy, expected):
    content = b"""<link rel=match href=ref.html>
<meta name=fuzzy content="%s">
""" % fuzzy

    s = create("foo/test.html", content)

    assert s.content_is_ref_node
    assert s.fuzzy == expected

@pytest.mark.parametrize("fuzzy, expected", [
    ([b"1;200"], {None: [[1, 1], [200, 200]]}),
    ([b"ref-2.html:0-1;100-200"], {("/foo/test.html", "/foo/ref-2.html", "=="): [[0, 1], [100, 200]]}),
    ([b"1;200", b"ref-2.html:0-1;100-200"],
     {None: [[1, 1], [200, 200]],
      ("/foo/test.html", "/foo/ref-2.html", "=="): [[0,1], [100, 200]]})])
def test_reftest_fuzzy_multi(fuzzy, expected):
    content = b"""<link rel=match href=ref-1.html>
<link rel=match href=ref-2.html>
"""
    for item in fuzzy:
        content += b'\n<meta name=fuzzy content="%s">' % item

    s = create("foo/test.html", content)

    assert s.content_is_ref_node
    assert s.fuzzy == expected

@pytest.mark.parametrize("pac, expected", [
    (b"proxy.pac", "proxy.pac")])
def test_pac(pac, expected):
    content = b"""
<meta name=pac content="%s">
""" % pac

    s = create("foo/test.html", content)
    assert s.pac == expected

@pytest.mark.parametrize("page_ranges, expected", [
    (b"1-2", [[1, 2]]),
    (b"1-1,3-4", [[1, 1], [3, 4]]),
    (b"1,3", [[1], [3]]),
    (b"2-", [[2, None]]),
    (b"-2", [[None, 2]]),
    (b"-2,2-", [[None, 2], [2, None]]),
    (b"1,6-7,8", [[1], [6, 7], [8]])])
def test_page_ranges(page_ranges, expected):
    content = b"""<link rel=match href=ref.html>
<meta name=reftest-pages content="%s">
""" % page_ranges

    s = create("foo/test-print.html", content)

    assert s.page_ranges == {"/foo/test-print.html": expected}


@pytest.mark.parametrize("page_ranges", [b"a", b"1-a", b"1=2", b"1-2:2-3"])
def test_page_ranges_invalid(page_ranges):
    content = b"""<link rel=match href=ref.html>
<meta name=reftest-pages content="%s">
""" % page_ranges

    s = create("foo/test-print.html", content)
    with pytest.raises(ValueError):
        s.page_ranges


def test_hash():
    s = SourceFile("/", "foo", "/", contents=b"Hello, World!")
    assert "b45ef6fec89518d314f546fd6c3025367b721684" == s.hash
