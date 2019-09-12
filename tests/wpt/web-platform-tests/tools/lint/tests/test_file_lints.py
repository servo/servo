from __future__ import unicode_literals

from ..lint import check_file_contents
from .base import check_errors
import os
import pytest
import six

INTERESTING_FILE_NAMES = {
    "python": [
        "test.py",
    ],
    "js": [
        "test.js",
    ],
    "web-lax": [
        "test.htm",
        "test.html",
    ],
    "web-strict": [
        "test.svg",
        "test.xht",
        "test.xhtml",
    ],
}

def check_with_files(input_bytes):
    return {
        filename: (check_file_contents("", filename, six.BytesIO(input_bytes)), kind)
        for (filename, kind) in
        (
            (os.path.join("html", filename), kind)
            for (kind, filenames) in INTERESTING_FILE_NAMES.items()
            for filename in filenames
        )
    }


def test_trailing_whitespace():
    error_map = check_with_files(b"test; ")

    for (filename, (errors, kind)) in error_map.items():
        check_errors(errors)

        expected = [("TRAILING WHITESPACE", "Whitespace at EOL", filename, 1)]
        if kind == "web-strict":
            expected.append(("PARSE-FAILED", "Unable to parse file", filename, None))
        assert errors == expected


def test_indent_tabs():
    error_map = check_with_files(b"def foo():\n\x09pass")

    for (filename, (errors, kind)) in error_map.items():
        check_errors(errors)

        expected = [("INDENT TABS", "Tabs used for indentation", filename, 2)]
        if kind == "web-strict":
            expected.append(("PARSE-FAILED", "Unable to parse file", filename, None))
        assert errors == expected


def test_cr_not_at_eol():
    error_map = check_with_files(b"line1\rline2\r")

    for (filename, (errors, kind)) in error_map.items():
        check_errors(errors)

        expected = [("CR AT EOL", "CR character in line separator", filename, 1)]
        if kind == "web-strict":
            expected.append(("PARSE-FAILED", "Unable to parse file", filename, None))
        assert errors == expected


def test_cr_at_eol():
    error_map = check_with_files(b"line1\r\nline2\r\n")

    for (filename, (errors, kind)) in error_map.items():
        check_errors(errors)

        expected = [
            ("CR AT EOL", "CR character in line separator", filename, 1),
            ("CR AT EOL", "CR character in line separator", filename, 2),
        ]
        if kind == "web-strict":
            expected.append(("PARSE-FAILED", "Unable to parse file", filename, None))
        assert errors == expected


def test_w3c_test_org():
    error_map = check_with_files(b"import('http://www.w3c-test.org/')")

    for (filename, (errors, kind)) in error_map.items():
        check_errors(errors)

        expected = [("W3C-TEST.ORG", "External w3c-test.org domain used", filename, 1)]
        if kind == "python":
            expected.append(("PARSE-FAILED", "Unable to parse file", filename, 1))
        elif kind == "web-strict":
            expected.append(("PARSE-FAILED", "Unable to parse file", filename, None))
        assert errors == expected

def test_web_platform_test():
    error_map = check_with_files(b"import('http://web-platform.test/')")

    for (filename, (errors, kind)) in error_map.items():
        check_errors(errors)

        expected = [("WEB-PLATFORM.TEST", "Internal web-platform.test domain used", filename, 1)]
        if kind == "python":
            expected.append(("PARSE-FAILED", "Unable to parse file", filename, 1))
        elif kind == "web-strict":
            expected.append(("PARSE-FAILED", "Unable to parse file", filename, None))
        assert errors == expected


def test_webidl2_js():
    error_map = check_with_files(b"<script src=/resources/webidl2.js>")

    for (filename, (errors, kind)) in error_map.items():
        check_errors(errors)

        expected = [("WEBIDL2.JS", "Legacy webidl2.js script used", filename, 1)]
        if kind == "python":
            expected.append(("PARSE-FAILED", "Unable to parse file", filename, 1))
        elif kind == "web-strict":
            expected.append(("PARSE-FAILED", "Unable to parse file", filename, None))
        assert errors == expected


def test_console():
    error_map = check_with_files(b"<script>\nconsole.log('error');\nconsole.error ('log')\n</script>")

    for (filename, (errors, kind)) in error_map.items():
        check_errors(errors)

        if kind in ["web-lax", "web-strict", "js"]:
            assert errors == [
                ("CONSOLE", "Console logging API used", filename, 2),
                ("CONSOLE", "Console logging API used", filename, 3),
            ]
        else:
            assert errors == [("PARSE-FAILED", "Unable to parse file", filename, 1)]


def test_setTimeout():
    error_map = check_with_files(b"<script>setTimeout(() => 1, 10)</script>")

    for (filename, (errors, kind)) in error_map.items():
        check_errors(errors)

        if kind == "python":
            assert errors == [("PARSE-FAILED", "Unable to parse file", filename, 1)]
        else:
            assert errors == [('SET TIMEOUT',
                               'setTimeout used; step_timeout should typically be used instead',
                               filename,
                               1)]


def test_eventSender():
    error_map = check_with_files(b"<script>eventSender.mouseDown()</script>")

    for (filename, (errors, kind)) in error_map.items():
        check_errors(errors)

        if kind == "python":
            assert errors == [("PARSE-FAILED", "Unable to parse file", filename, 1)]
        else:
            assert errors == [('LAYOUTTESTS APIS',
                               'eventSender/testRunner/internals used; these are LayoutTests-specific APIs (WebKit/Blink)',
                               filename,
                               1)]


def test_testRunner():
    error_map = check_with_files(b"<script>if (window.testRunner) { testRunner.waitUntilDone(); }</script>")

    for (filename, (errors, kind)) in error_map.items():
        check_errors(errors)

        if kind == "python":
            assert errors == [("PARSE-FAILED", "Unable to parse file", filename, 1)]
        else:
            assert errors == [('LAYOUTTESTS APIS',
                               'eventSender/testRunner/internals used; these are LayoutTests-specific APIs (WebKit/Blink)',
                               filename,
                               1)]


def test_internals():
    error_map = check_with_files(b"<script>if (window.internals) { internals.doAThing(); }</script>")

    for (filename, (errors, kind)) in error_map.items():
        check_errors(errors)

        if kind == "python":
            assert errors == [("PARSE-FAILED", "Unable to parse file", filename, 1)]
        else:
            assert errors == [('LAYOUTTESTS APIS',
                               'eventSender/testRunner/internals used; these are LayoutTests-specific APIs (WebKit/Blink)',
                               filename,
                               1)]


def test_missing_deps():
    error_map = check_with_files(b"<script src='/gen/foo.js'></script>")

    for (filename, (errors, kind)) in error_map.items():
        check_errors(errors)

        if kind == "python":
            assert errors == [("PARSE-FAILED", "Unable to parse file", filename, 1)]
        else:
            assert errors == [('MISSING DEPENDENCY',
                               'Chromium-specific content referenced',
                               filename,
                               1)]


def test_no_missing_deps():
    error_map = check_with_files(b"""<head>
<script src='/foo/gen/foo.js'></script>
<script src='/gens/foo.js'></script>
</head>""")

    for (filename, (errors, kind)) in error_map.items():
        check_errors(errors)

        if kind == "python":
            assert errors == [("PARSE-FAILED", "Unable to parse file", filename, 1)]
        else:
            assert errors == []


def test_meta_timeout():
    code = b"""
<html xmlns="http://www.w3.org/1999/xhtml">
<meta name="timeout" />
<meta name="timeout" content="short" />
<meta name="timeout" content="long" />
</html>
"""
    error_map = check_with_files(code)

    for (filename, (errors, kind)) in error_map.items():
        check_errors(errors)

        if kind in ["web-lax", "web-strict"]:
            assert errors == [
                ("MULTIPLE-TIMEOUT", "More than one meta name='timeout'", filename, None),
                ("INVALID-TIMEOUT", "Invalid timeout value ", filename, None),
                ("INVALID-TIMEOUT", "Invalid timeout value short", filename, None),
            ]
        elif kind == "python":
            assert errors == [
                ("PARSE-FAILED", "Unable to parse file", filename, 2),
            ]


def test_early_testharnessreport():
    code = b"""
<html xmlns="http://www.w3.org/1999/xhtml">
<script src="/resources/testharnessreport.js"></script>
<script src="/resources/testharness.js"></script>
</html>
"""
    error_map = check_with_files(code)

    for (filename, (errors, kind)) in error_map.items():
        check_errors(errors)

        if kind in ["web-lax", "web-strict"]:
            assert errors == [
                ("EARLY-TESTHARNESSREPORT", "testharnessreport.js script seen before testharness.js script", filename, None),
            ]
        elif kind == "python":
            assert errors == [
                ("PARSE-FAILED", "Unable to parse file", filename, 2),
            ]


def test_multiple_testharness():
    code = b"""
<html xmlns="http://www.w3.org/1999/xhtml">
<script src="/resources/testharness.js"></script>
<script src="/resources/testharness.js"></script>
</html>
"""
    error_map = check_with_files(code)

    for (filename, (errors, kind)) in error_map.items():
        check_errors(errors)

        if kind in ["web-lax", "web-strict"]:
            assert errors == [
                ("MULTIPLE-TESTHARNESS", "More than one <script src='/resources/testharness.js'>", filename, None),
                ("MISSING-TESTHARNESSREPORT", "Missing <script src='/resources/testharnessreport.js'>", filename, None),
            ]
        elif kind == "python":
            assert errors == [
                ("PARSE-FAILED", "Unable to parse file", filename, 2),
            ]


def test_multiple_testharnessreport():
    code = b"""
<html xmlns="http://www.w3.org/1999/xhtml">
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
<script src="/resources/testharnessreport.js"></script>
</html>
"""
    error_map = check_with_files(code)

    for (filename, (errors, kind)) in error_map.items():
        check_errors(errors)

        if kind in ["web-lax", "web-strict"]:
            assert errors == [
                ("MULTIPLE-TESTHARNESSREPORT", "More than one <script src='/resources/testharnessreport.js'>", filename, None),
            ]
        elif kind == "python":
            assert errors == [
                ("PARSE-FAILED", "Unable to parse file", filename, 2),
            ]


def test_multiple_testdriver():
    code = b"""
<html xmlns="http://www.w3.org/1999/xhtml">
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
<script src="/resources/testdriver.js"></script>
<script src="/resources/testdriver.js"></script>
<script src="/resources/testdriver-vendor.js"></script>
</html>
"""
    error_map = check_with_files(code)

    for (filename, (errors, kind)) in error_map.items():
        check_errors(errors)

        if kind in ["web-lax", "web-strict"]:
            assert errors == [
                ("MULTIPLE-TESTDRIVER", "More than one <script src='/resources/testdriver.js'>", filename, None),
            ]
        elif kind == "python":
            assert errors == [
                ("PARSE-FAILED", "Unable to parse file", filename, 2),
            ]


def test_multiple_testdriver_vendor():
    code = b"""
<html xmlns="http://www.w3.org/1999/xhtml">
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
<script src="/resources/testdriver.js"></script>
<script src="/resources/testdriver-vendor.js"></script>
<script src="/resources/testdriver-vendor.js"></script>
</html>
"""
    error_map = check_with_files(code)

    for (filename, (errors, kind)) in error_map.items():
        check_errors(errors)

        if kind in ["web-lax", "web-strict"]:
            assert errors == [
                ("MULTIPLE-TESTDRIVER-VENDOR", "More than one <script src='/resources/testdriver-vendor.js'>", filename, None),
            ]
        elif kind == "python":
            assert errors == [
                ("PARSE-FAILED", "Unable to parse file", filename, 2),
            ]


def test_missing_testdriver_vendor():
    code = b"""
<html xmlns="http://www.w3.org/1999/xhtml">
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
<script src="/resources/testdriver.js"></script>
</html>
"""
    error_map = check_with_files(code)

    for (filename, (errors, kind)) in error_map.items():
        check_errors(errors)

        if kind in ["web-lax", "web-strict"]:
            assert errors == [
                ("MISSING-TESTDRIVER-VENDOR", "Missing <script src='/resources/testdriver-vendor.js'>", filename, None),
            ]
        elif kind == "python":
            assert errors == [
                ("PARSE-FAILED", "Unable to parse file", filename, 2),
            ]


def test_present_testharnesscss():
    code = b"""
<html xmlns="http://www.w3.org/1999/xhtml">
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
<link rel="stylesheet" href="/resources/testharness.css"/>
</html>
"""
    error_map = check_with_files(code)

    for (filename, (errors, kind)) in error_map.items():
        check_errors(errors)

        if kind in ["web-lax", "web-strict"]:
            assert errors == [
                ("PRESENT-TESTHARNESSCSS", "Explicit link to testharness.css present", filename, None),
            ]
        elif kind == "python":
            assert errors == [
                ("PARSE-FAILED", "Unable to parse file", filename, 2),
            ]


def test_testharness_path():
    code = b"""\
<html xmlns="http://www.w3.org/1999/xhtml">
<script src="testharness.js"></script>
<script src="resources/testharness.js"></script>
<script src="../resources/testharness.js"></script>
<script src="http://w3c-test.org/resources/testharness.js"></script>
</html>
"""
    error_map = check_with_files(code)

    for (filename, (errors, kind)) in error_map.items():
        check_errors(errors)

        expected = [("W3C-TEST.ORG", "External w3c-test.org domain used", filename, 5)]
        if kind == "python":
            expected.append(("PARSE-FAILED", "Unable to parse file", filename, 1))
        elif kind in ["web-lax", "web-strict"]:
            expected.extend([
                ("TESTHARNESS-PATH", "testharness.js script seen with incorrect path", filename, None),
                ("TESTHARNESS-PATH", "testharness.js script seen with incorrect path", filename, None),
                ("TESTHARNESS-PATH", "testharness.js script seen with incorrect path", filename, None),
                ("TESTHARNESS-PATH", "testharness.js script seen with incorrect path", filename, None),
            ])
        assert errors == expected


def test_testharnessreport_path():
    code = b"""\
<html xmlns="http://www.w3.org/1999/xhtml">
<script src="testharnessreport.js"></script>
<script src="resources/testharnessreport.js"></script>
<script src="../resources/testharnessreport.js"></script>
<script src="http://w3c-test.org/resources/testharnessreport.js"></script>
</html>
"""
    error_map = check_with_files(code)

    for (filename, (errors, kind)) in error_map.items():
        check_errors(errors)

        expected = [("W3C-TEST.ORG", "External w3c-test.org domain used", filename, 5)]
        if kind == "python":
            expected.append(("PARSE-FAILED", "Unable to parse file", filename, 1))
        elif kind in ["web-lax", "web-strict"]:
            expected.extend([
                ("TESTHARNESSREPORT-PATH", "testharnessreport.js script seen with incorrect path", filename, None),
                ("TESTHARNESSREPORT-PATH", "testharnessreport.js script seen with incorrect path", filename, None),
                ("TESTHARNESSREPORT-PATH", "testharnessreport.js script seen with incorrect path", filename, None),
                ("TESTHARNESSREPORT-PATH", "testharnessreport.js script seen with incorrect path", filename, None),
            ])
        assert errors == expected


def test_testdriver_path():
    code = b"""\
<html xmlns="http://www.w3.org/1999/xhtml">
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
<script src="testdriver.js"></script>
<script src="/elsewhere/testdriver.js"></script>
<script src="/elsewhere/resources/testdriver.js"></script>
<script src="/resources/elsewhere/testdriver.js"></script>
<script src="../resources/testdriver.js"></script>
<script src="/resources/testdriver-vendor.js"></script>
</html>
"""
    error_map = check_with_files(code)

    for (filename, (errors, kind)) in error_map.items():
        check_errors(errors)

        expected = []
        if kind == "python":
            expected.append(("PARSE-FAILED", "Unable to parse file", filename, 1))
        elif kind in ["web-lax", "web-strict"]:
            expected.extend([
                ("TESTDRIVER-PATH", "testdriver.js script seen with incorrect path", filename, None),
                ("TESTDRIVER-PATH", "testdriver.js script seen with incorrect path", filename, None),
                ("TESTDRIVER-PATH", "testdriver.js script seen with incorrect path", filename, None),
                ("TESTDRIVER-PATH", "testdriver.js script seen with incorrect path", filename, None),
                ("TESTDRIVER-PATH", "testdriver.js script seen with incorrect path", filename, None)
            ])
        assert errors == expected


def test_testdriver_vendor_path():
    code = b"""\
<html xmlns="http://www.w3.org/1999/xhtml">
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
<script src="/resources/testdriver.js"></script>
<script src="testdriver-vendor.js"></script>
<script src="/elsewhere/testdriver-vendor.js"></script>
<script src="/elsewhere/resources/testdriver-vendor.js"></script>
<script src="/resources/elsewhere/testdriver-vendor.js"></script>
<script src="../resources/testdriver-vendor.js"></script>
</html>
"""
    error_map = check_with_files(code)

    for (filename, (errors, kind)) in error_map.items():
        check_errors(errors)

        if kind == "python":
            expected = set([("PARSE-FAILED", "Unable to parse file", filename, 1)])
        elif kind in ["web-lax", "web-strict"]:
            expected = set([
                ("MISSING-TESTDRIVER-VENDOR", "Missing <script src='/resources/testdriver-vendor.js'>", filename, None),
                ("TESTDRIVER-VENDOR-PATH", "testdriver-vendor.js script seen with incorrect path", filename, None),
                ("TESTDRIVER-VENDOR-PATH", "testdriver-vendor.js script seen with incorrect path", filename, None),
                ("TESTDRIVER-VENDOR-PATH", "testdriver-vendor.js script seen with incorrect path", filename, None),
                ("TESTDRIVER-VENDOR-PATH", "testdriver-vendor.js script seen with incorrect path", filename, None),
                ("TESTDRIVER-VENDOR-PATH", "testdriver-vendor.js script seen with incorrect path", filename, None)
            ])
        else:
            expected = set()

        assert set(errors) == expected


def test_not_testharness_path():
    code = b"""\
<html xmlns="http://www.w3.org/1999/xhtml">
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
<script src="resources/webperftestharness.js"></script>
<script src="/resources/testdriver.js"></script>
<script src="/resources/testdriver-vendor.js"></script>
</html>
"""
    error_map = check_with_files(code)

    for (filename, (errors, kind)) in error_map.items():
        check_errors(errors)

        if kind == "python":
            assert errors == [
                ("PARSE-FAILED", "Unable to parse file", filename, 1),
            ]
        else:
            assert errors == []


def test_variant_missing():
    code = b"""\
<html xmlns="http://www.w3.org/1999/xhtml">
<meta name="variant">
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
</html>
"""
    error_map = check_with_files(code)

    for (filename, (errors, kind)) in error_map.items():
        check_errors(errors)

        if kind == "python":
            assert errors == [
                ("PARSE-FAILED", "Unable to parse file", filename, 1),
            ]
        elif kind == "web-lax":
            assert errors == [
                ("VARIANT-MISSING", "<meta name=variant> missing 'content' attribute", filename, None)
            ]


# A corresponding "positive" test cannot be written because the manifest
# SourceFile implementation raises a runtime exception for the condition this
# linting rule describes
@pytest.mark.parametrize("content", ["",
                                     "?"
                                     "#"])
def test_variant_malformed_negative(content):
    code = """\
<html xmlns="http://www.w3.org/1999/xhtml">
<meta name="variant" content="{}">
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
</html>
""".format(content).encode("utf-8")
    error_map = check_with_files(code)

    for (filename, (errors, kind)) in error_map.items():
        check_errors(errors)

        if kind == "python":
            assert errors == [
                ("PARSE-FAILED", "Unable to parse file", filename, 1),
            ]
        elif kind == "web-lax":
            assert errors == []


def test_late_timeout():
    code = b"""\
<html xmlns="http://www.w3.org/1999/xhtml">
<script src="/resources/testharness.js"></script>
<meta name="timeout" content="long">
<script src="/resources/testharnessreport.js"></script>
</html>
"""
    error_map = check_with_files(code)

    for (filename, (errors, kind)) in error_map.items():
        check_errors(errors)

        if kind == "python":
            assert errors == [
                ("PARSE-FAILED", "Unable to parse file", filename, 1),
            ]
        elif kind == "web-lax":
            assert errors == [
                ("LATE-TIMEOUT", "<meta name=timeout> seen after testharness.js script", filename, None)
            ]


@pytest.mark.skipif(six.PY3, reason="Cannot parse print statements from python 3")
def test_print_statement():
    error_map = check_with_files(b"def foo():\n  print 'statement'\n  print\n")

    for (filename, (errors, kind)) in error_map.items():
        check_errors(errors)

        if kind == "python":
            assert errors == [
                ("PRINT STATEMENT", "Print function used", filename, 2),
                ("PRINT STATEMENT", "Print function used", filename, 3),
            ]
        elif kind == "web-strict":
            assert errors == [
                ("PARSE-FAILED", "Unable to parse file", filename, None),
            ]
        else:
            assert errors == []


def test_print_function():
    error_map = check_with_files(b"def foo():\n  print('function')\n")

    for (filename, (errors, kind)) in error_map.items():
        check_errors(errors)

        if kind == "python":
            assert errors == [
                ("PRINT STATEMENT", "Print function used", filename, 2),
            ]
        elif kind == "web-strict":
            assert errors == [
                ("PARSE-FAILED", "Unable to parse file", filename, None),
            ]
        else:
            assert errors == []


def test_ahem_system_font():
    code = b"""\
<html>
<style>
body {
  font-family: aHEm, sans-serif;
}
</style>
</html>
"""
    error_map = check_with_files(code)
    for (filename, (errors, kind)) in error_map.items():
        check_errors(errors)

        if filename.endswith((".htm", ".html", ".xht", ".xhtml")):
            assert errors == [
                ("AHEM SYSTEM FONT", "Don't use Ahem as a system font, use /fonts/ahem.css", filename, None)
            ]


def test_ahem_web_font():
    code = b"""\
<html>
<link rel="stylesheet" type="text/css" href="/fonts/ahem.css" />
<style>
body {
  font-family: aHEm, sans-serif;
}
</style>
</html>
"""
    error_map = check_with_files(code)
    for (filename, (errors, kind)) in error_map.items():
        check_errors(errors)

        if filename.endswith((".htm", ".html", ".xht", ".xhtml")):
            assert errors == []


open_mode_code = """
def first():
    return {0}("test.png")

def second():
    return {0}("test.png", "r")

def third():
    return {0}("test.png", "rb")

def fourth():
    return {0}("test.png", encoding="utf-8")

def fifth():
    return {0}("test.png", mode="rb")
"""


def test_open_mode():
    for method in ["open", "file"]:
        code = open_mode_code.format(method).encode("utf-8")
        errors = check_file_contents("", "test.py", six.BytesIO(code))
        check_errors(errors)

        message = ("File opened without providing an explicit mode (note: " +
                   "binary files must be read with 'b' in the mode flags)")

        assert errors == [
            ("OPEN-NO-MODE", message, "test.py", 3),
            ("OPEN-NO-MODE", message, "test.py", 12),
        ]


@pytest.mark.parametrize(
    "filename,expect_error",
    [
        ("foo/bar.html", False),
        ("css/bar.html", True),
    ])
def test_css_support_file(filename, expect_error):
    errors = check_file_contents("", filename, six.BytesIO(b""))
    check_errors(errors)

    if expect_error:
        assert errors == [
            ('SUPPORT-WRONG-DIR',
             'Support file not in support directory',
             filename,
             None),
        ]
    else:
        assert errors == []


def test_css_missing_file_in_css():
    code = b"""\
<html xmlns="http://www.w3.org/1999/xhtml">
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
</html>
"""
    errors = check_file_contents("", "css/foo/bar.html", six.BytesIO(code))
    check_errors(errors)

    assert errors == [
        ('MISSING-LINK',
         'Testcase file must have a link to a spec',
         "css/foo/bar.html",
         None),
    ]


def test_css_missing_file_manual():
    errors = check_file_contents("", "css/foo/bar-manual.html", six.BytesIO(b""))
    check_errors(errors)

    assert errors == [
        ('MISSING-LINK',
         'Testcase file must have a link to a spec',
         "css/foo/bar-manual.html",
         None),
    ]


@pytest.mark.parametrize("filename", [
    "foo.worker.js",
    "foo.any.js",
])
@pytest.mark.parametrize("input,error", [
    (b"""//META: title=foo\n""", None),
    (b"""//META: timeout=long\n""", None),
    (b"""// META: timeout=long\n""", None),
    (b"""//  META: timeout=long\n""", None),
    (b"""// META: script=foo.js\n""", None),
    (b"""// META: variant=\n""", None),
    (b"""// META: variant=?wss\n""", None),
    (b"""# META:\n""", None),
    (b"""\n// META: timeout=long\n""", (2, "STRAY-METADATA")),
    (b""" // META: timeout=long\n""", (1, "INDENTED-METADATA")),
    (b"""// META: timeout=long\n// META: timeout=long\n""", None),
    (b"""// META: timeout=long\n\n// META: timeout=long\n""", (3, "STRAY-METADATA")),
    (b"""// META: timeout=long\n// Start of the test\n// META: timeout=long\n""", (3, "STRAY-METADATA")),
    (b"""// META:\n""", (1, "BROKEN-METADATA")),
    (b"""// META: foobar\n""", (1, "BROKEN-METADATA")),
    (b"""// META: foo=bar\n""", (1, "UNKNOWN-METADATA")),
    (b"""// META: timeout=bar\n""", (1, "UNKNOWN-TIMEOUT-METADATA")),
])
def test_script_metadata(filename, input, error):
    errors = check_file_contents("", filename, six.BytesIO(input))
    check_errors(errors)

    if error is not None:
        line, kind = error
        messages = {
            "STRAY-METADATA": "Metadata comments should start the file",
            "INDENTED-METADATA": "Metadata comments should start the line",
            "BROKEN-METADATA": "Metadata comment is not formatted correctly",
            "UNKNOWN-TIMEOUT-METADATA": "Unexpected value for timeout metadata",
            "UNKNOWN-METADATA": "Unexpected kind of metadata",
        }
        assert errors == [
            (kind,
             messages[kind],
             filename,
             line),
        ]
    else:
        assert errors == []


@pytest.mark.parametrize("globals,error", [
    (b"", None),
    (b"default", None),
    (b"!default", None),
    (b"window", None),
    (b"!window", None),
    (b"!dedicatedworker", None),
    (b"window, !window", "BROKEN-GLOBAL-METADATA"),
    (b"!serviceworker", "BROKEN-GLOBAL-METADATA"),
    (b"serviceworker, !serviceworker", "BROKEN-GLOBAL-METADATA"),
    (b"worker, !dedicatedworker", None),
    (b"worker, !serviceworker", None),
    (b"!worker", None),
    (b"foo", "UNKNOWN-GLOBAL-METADATA"),
    (b"!foo", "UNKNOWN-GLOBAL-METADATA"),
])
def test_script_globals_metadata(globals, error):
    filename = "foo.any.js"
    input = b"""// META: global=%s\n""" % globals
    errors = check_file_contents("", filename, six.BytesIO(input))
    check_errors(errors)

    if error is not None:
        errors = [(k, f, l) for (k, _, f, l) in errors]
        assert errors == [
            (error,
             filename,
             1),
        ]
    else:
        assert errors == []


@pytest.mark.parametrize("input,error", [
    (b"""#META: timeout=long\n""", None),
    (b"""# META: timeout=long\n""", None),
    (b"""#  META: timeout=long\n""", None),
    (b""""// META:"\n""", None),
    (b"""\n# META: timeout=long\n""", (2, "STRAY-METADATA")),
    (b""" # META: timeout=long\n""", (1, "INDENTED-METADATA")),
    (b"""# META: timeout=long\n# META: timeout=long\n""", None),
    (b"""# META: timeout=long\n\n# META: timeout=long\n""", (3, "STRAY-METADATA")),
    (b"""# META: timeout=long\n# Start of the test\n# META: timeout=long\n""", (3, "STRAY-METADATA")),
    (b"""# META:\n""", (1, "BROKEN-METADATA")),
    (b"""# META: foobar\n""", (1, "BROKEN-METADATA")),
    (b"""# META: foo=bar\n""", (1, "UNKNOWN-METADATA")),
    (b"""# META: timeout=bar\n""", (1, "UNKNOWN-TIMEOUT-METADATA")),
])
def test_python_metadata(input, error):
    filename = "test.py"
    errors = check_file_contents("", filename, six.BytesIO(input))
    check_errors(errors)

    if error is not None:
        line, kind = error
        messages = {
            "STRAY-METADATA": "Metadata comments should start the file",
            "INDENTED-METADATA": "Metadata comments should start the line",
            "BROKEN-METADATA": "Metadata comment is not formatted correctly",
            "UNKNOWN-TIMEOUT-METADATA": "Unexpected value for timeout metadata",
            "UNKNOWN-METADATA": "Unexpected kind of metadata",
        }
        assert errors == [
            (kind,
             messages[kind],
             filename,
             line),
        ]
    else:
        assert errors == []
