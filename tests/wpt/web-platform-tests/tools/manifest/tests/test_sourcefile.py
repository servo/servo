from ..sourcefile import SourceFile

def create(filename, contents=b""):
    assert isinstance(contents, bytes)
    return SourceFile("/", filename, "/", contents=contents)


def items(s):
    return [
        (item.item_type, item.url)
        for item in s.manifest_items()
    ]


def test_name_is_non_test():
    non_tests = [
        ".gitignore",
        ".travis.yml",
        "MANIFEST.json",
        "tools/test.html",
        "resources/test.html",
        "common/test.html",
        "conformance-checkers/test.html",
    ]

    for rel_path in non_tests:
        s = create(rel_path)
        assert s.name_is_non_test

        assert not s.content_is_testharness

        assert items(s) == []


def test_name_is_manual():
    manual_tests = [
        "html/test-manual.html",
        "html/test-manual.xhtml",
    ]

    for rel_path in manual_tests:
        s = create(rel_path)
        assert not s.name_is_non_test
        assert s.name_is_manual

        assert not s.content_is_testharness

        assert items(s) == [("manual", "/" + rel_path)]


def test_worker():
    s = create("html/test.worker.js")
    assert not s.name_is_non_test
    assert not s.name_is_manual
    assert not s.name_is_multi_global
    assert s.name_is_worker
    assert not s.name_is_reference

    assert not s.content_is_testharness

    assert items(s) == [("testharness", "/html/test.worker")]


def test_multi_global():
    s = create("html/test.any.js")
    assert not s.name_is_non_test
    assert not s.name_is_manual
    assert s.name_is_multi_global
    assert not s.name_is_worker
    assert not s.name_is_reference

    assert not s.content_is_testharness

    assert items(s) == [
        ("testharness", "/html/test.any.html"),
        ("testharness", "/html/test.any.worker"),
    ]


def test_testharness():
    content = b"<script src=/resources/testharness.js></script>"

    for ext in ["htm", "html"]:
        filename = "html/test." + ext
        s = create(filename, content)

        assert not s.name_is_non_test
        assert not s.name_is_manual
        assert not s.name_is_multi_global
        assert not s.name_is_worker
        assert not s.name_is_reference

        assert s.content_is_testharness

        assert items(s) == [("testharness", "/" + filename)]


def test_relative_testharness():
    content = b"<script src=../resources/testharness.js></script>"

    for ext in ["htm", "html"]:
        filename = "html/test." + ext
        s = create(filename, content)

        assert not s.name_is_non_test
        assert not s.name_is_manual
        assert not s.name_is_multi_global
        assert not s.name_is_worker
        assert not s.name_is_reference

        assert not s.content_is_testharness

        assert items(s) == []


def test_testharness_xhtml():
    content = b"""
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
</head>
<body/>
</html>
"""

    for ext in ["xhtml", "xht", "xml"]:
        filename = "html/test." + ext
        s = create(filename, content)

        assert not s.name_is_non_test
        assert not s.name_is_manual
        assert not s.name_is_multi_global
        assert not s.name_is_worker
        assert not s.name_is_reference

        assert s.content_is_testharness

        assert items(s) == [("testharness", "/" + filename)]


def test_relative_testharness_xhtml():
    content = b"""
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
<script src="../resources/testharness.js"></script>
<script src="../resources/testharnessreport.js"></script>
</head>
<body/>
</html>
"""

    for ext in ["xhtml", "xht", "xml"]:
        filename = "html/test." + ext
        s = create(filename, content)

        assert not s.name_is_non_test
        assert not s.name_is_manual
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
    assert not s.name_is_multi_global
    assert not s.name_is_worker
    assert not s.name_is_reference

    assert s.root
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
    assert not s.name_is_multi_global
    assert not s.name_is_worker
    assert not s.name_is_reference

    assert s.root
    assert not s.content_is_testharness

    assert items(s) == []


def test_testharness_ext():
    content = b"<script src=/resources/testharness.js></script>"

    for filename in ["test", "test.test"]:
        s = create("html/" + filename, content)

        assert not s.name_is_non_test
        assert not s.name_is_manual
        assert not s.name_is_multi_global
        assert not s.name_is_worker
        assert not s.name_is_reference

        assert not s.root
        assert not s.content_is_testharness

        assert items(s) == []
