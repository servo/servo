# mypy: allow-untyped-defs

import builtins
import io
import logging
import os
import pickle
import platform
from unittest.mock import MagicMock, patch
from typing import Generator, Tuple

import pytest

import localpaths  # type: ignore
from . import serve
from .serve import (
    ConfigBuilder,
    Test262WindowHandler,
    Test262WindowTestHandler,
    Test262WindowModuleHandler,
    Test262WindowModuleTestHandler,
    Test262StrictWindowHandler,
    Test262StrictWindowTestHandler,
    Test262StrictHandler,
    inject_script)


logger = logging.getLogger()

@pytest.mark.skipif(platform.uname()[0] == "Windows",
                    reason="Expected contents are platform-dependent")
def test_make_hosts_file_nix():
    with ConfigBuilder(logger,
                       ports={"http": [8000]},
                       browser_host="foo.bar",
                       alternate_hosts={"alt": "foo2.bar"},
                       subdomains={"a", "b"},
                       not_subdomains={"x, y"}) as c:
        hosts = serve.make_hosts_file(c, "192.168.42.42")
        lines = hosts.split("\n")
        assert lines == [
            "# Start web-platform-tests hosts",
            "192.168.42.42\tfoo.bar",
            "192.168.42.42\ta.foo.bar",
            "192.168.42.42\tb.foo.bar",
            "192.168.42.42\tfoo2.bar",
            "192.168.42.42\ta.foo2.bar",
            "192.168.42.42\tb.foo2.bar",
            "# End web-platform-tests hosts",
            "",
        ]


@pytest.mark.skipif(platform.uname()[0] != "Windows",
                    reason="Expected contents are platform-dependent")
def test_make_hosts_file_windows():
    with ConfigBuilder(logger,
                       ports={"http": [8000]},
                       browser_host="foo.bar",
                       alternate_hosts={"alt": "foo2.bar"},
                       subdomains={"a", "b"},
                       not_subdomains={"x", "y"}) as c:
        hosts = serve.make_hosts_file(c, "192.168.42.42")
        lines = hosts.split("\n")
        assert lines == [
            "# Start web-platform-tests hosts",
            "192.168.42.42\tfoo.bar",
            "192.168.42.42\ta.foo.bar",
            "192.168.42.42\tb.foo.bar",
            "192.168.42.42\tfoo2.bar",
            "192.168.42.42\ta.foo2.bar",
            "192.168.42.42\tb.foo2.bar",
            "0.0.0.0\tx.foo.bar",
            "0.0.0.0\ty.foo.bar",
            "0.0.0.0\tx.foo2.bar",
            "0.0.0.0\ty.foo2.bar",
            "# End web-platform-tests hosts",
            "",
        ]


def test_ws_doc_root_default():
    with ConfigBuilder(logger) as c:
        assert c.doc_root == localpaths.repo_root
        assert c.ws_doc_root == os.path.join(localpaths.repo_root, "websockets", "handlers")
        assert c.paths["ws_doc_root"] == c.ws_doc_root


def test_init_ws_doc_root():
    with ConfigBuilder(logger, ws_doc_root="/") as c:
        assert c.doc_root == localpaths.repo_root  # check this hasn't changed
        assert c.ws_doc_root == "/"
        assert c.paths["ws_doc_root"] == c.ws_doc_root


def test_set_ws_doc_root():
    cb = ConfigBuilder(logger)
    cb.ws_doc_root = "/"
    with cb as c:
        assert c.doc_root == localpaths.repo_root  # check this hasn't changed
        assert c.ws_doc_root == "/"
        assert c.paths["ws_doc_root"] == c.ws_doc_root


def test_pickle():
    # Ensure that the config object can be pickled
    with ConfigBuilder(logger) as c:
        pickle.dumps(c)


def test_alternate_host_unspecified():
    ConfigBuilder(logger, browser_host="web-platform.test")


@pytest.mark.parametrize("primary, alternate", [
    ("web-platform.test", "web-platform.test"),
    ("a.web-platform.test", "web-platform.test"),
    ("web-platform.test", "a.web-platform.test"),
    ("a.web-platform.test", "a.web-platform.test"),
])
def test_alternate_host_invalid(primary, alternate):
    with pytest.raises(ValueError):
        ConfigBuilder(logger, browser_host=primary, alternate_hosts={"alt": alternate})

@pytest.mark.parametrize("primary, alternate", [
    ("web-platform.test", "not-web-platform.test"),
    ("a.web-platform.test", "b.web-platform.test"),
    ("web-platform-tests.dev", "web-platform-tests.live"),
])
def test_alternate_host_valid(primary, alternate):
    ConfigBuilder(logger, browser_host=primary, alternate_hosts={"alt": alternate})


# A token marking the location of expected script injection.
INJECT_SCRIPT_MARKER = b"<!-- inject here -->"


def test_inject_script_after_head():
    html = b"""<!DOCTYPE html>
    <html>
        <head>
        <!-- inject here --><script src="test.js"></script>
        </head>
        <body>
        </body>
    </html>"""
    assert INJECT_SCRIPT_MARKER in html
    assert inject_script(html.replace(INJECT_SCRIPT_MARKER, b""), INJECT_SCRIPT_MARKER) == html


def test_inject_script_no_html_head():
    html = b"""<!DOCTYPE html>
    <!-- inject here --><div></div>"""
    assert INJECT_SCRIPT_MARKER in html
    assert inject_script(html.replace(INJECT_SCRIPT_MARKER, b""), INJECT_SCRIPT_MARKER) == html


def test_inject_script_no_doctype():
    html = b"""<!-- inject here --><div></div>"""
    assert INJECT_SCRIPT_MARKER in html
    assert inject_script(html.replace(INJECT_SCRIPT_MARKER, b""), INJECT_SCRIPT_MARKER) == html


def test_inject_script_parse_error():
    html = b"""<!--<!-- inject here --><div></div>"""
    assert INJECT_SCRIPT_MARKER in html
    # On a parse error, the script should not be injected and the original content should be
    # returned.
    assert INJECT_SCRIPT_MARKER not in inject_script(html.replace(INJECT_SCRIPT_MARKER, b""), INJECT_SCRIPT_MARKER)


@pytest.fixture
def handler_setup() -> Generator[Tuple[str, str], None, None]:
    """Provides a mocked filesystem environment for testing any handler class."""
    tests_root = os.path.abspath(os.path.join(os.path.dirname(__file__), "tests", "testdata"))
    url_base = "/"

    mock_file_contents = {
        os.path.normpath(os.path.join(tests_root, "test262", "basic.js")): """/*---\ndescription: A basic test
includes: [assert.js, sta.js]
---*/
assert.sameValue(1, 1);
""",
        os.path.normpath(os.path.join(tests_root, "test262", "negative.js")): """/*---\ndescription: A negative test
negative:
  phase: runtime
  type: TypeError
---*/
throw new TypeError();
""",
        os.path.normpath(os.path.join(tests_root, "test262", "module.js")): """/*---\ndescription: A module test
flags: [module]
---*/
import {} from 'some-module';
""",
        os.path.normpath(os.path.join(tests_root, "test262", "async.js")): """/*---
description: An async test
flags: [async]
---*/
print('Test262:AsyncTestComplete');
""",
        os.path.normpath(os.path.join(tests_root, "test262", "teststrict.js")): """/*---\ndescription: A strict mode test
flags: [onlyStrict]
includes: [propertyHelper.js]
---*/
console.log('hello');
"""
    }

    # Store original functions to be called if our mock doesn't handle the file
    original_open = builtins.open
    original_exists = os.path.exists
    original_isdir = os.path.isdir

    def custom_open(file, mode='r', *args, **kwargs):
        normalized_file = os.path.normpath(file)
        if normalized_file in mock_file_contents:
            if 'b' in mode:
                return io.BytesIO(mock_file_contents[normalized_file].encode('utf-8'))
            else:
                return io.StringIO(mock_file_contents[normalized_file])
        return original_open(file, mode, *args, **kwargs)

    def custom_exists(path):
        normalized_path = os.path.normpath(path)
        return normalized_path in mock_file_contents or original_exists(path)

    def custom_isdir(path):
        normalized_path = os.path.normpath(path)
        expected_dir = os.path.normpath(os.path.join(tests_root, "test262"))
        return normalized_path == expected_dir or original_isdir(path)

    with patch('builtins.open', side_effect=custom_open), \
         patch('os.path.exists', side_effect=custom_exists), \
         patch('os.path.isdir', side_effect=custom_isdir):
        yield tests_root, url_base


def _create_mock_request(path: str) -> MagicMock:
    mock_request = MagicMock()
    mock_request.url_parts.path = path
    mock_request.url_parts.query = ""
    return mock_request


# Ensure pytest doesn't consider Test-prefixed classes to be testcases
Test262WindowHandler.__test__ = False  # type: ignore[attr-defined]
Test262WindowTestHandler.__test__ = False  # type: ignore[attr-defined]
Test262WindowModuleHandler.__test__ = False  # type: ignore[attr-defined]
Test262WindowModuleTestHandler.__test__ = False  # type: ignore[attr-defined]
Test262StrictWindowHandler.__test__ = False  # type: ignore[attr-defined]
Test262StrictWindowTestHandler.__test__ = False  # type: ignore[attr-defined]
Test262StrictHandler.__test__ = False  # type: ignore[attr-defined]

@pytest.mark.parametrize("handler_cls, expected", [
    (Test262WindowHandler, [(".test262.html", ".js", ".test262-test.html")]),
    (Test262WindowTestHandler, [(".test262-test.html", ".js")]),
    (Test262WindowModuleHandler, [(".test262-module.html", ".js", ".test262-module-test.html")]),
    (Test262WindowModuleTestHandler, [(".test262-module-test.html", ".js")]),
    (Test262StrictWindowHandler, [(".test262.strict.html", ".js", ".test262-test.strict.html")]),
    (Test262StrictWindowTestHandler, [(".test262-test.strict.html", ".js", ".test262.strict.js")]),
])
def test_path_replace(handler_setup, handler_cls, expected):
    """Verifies that handlers correctly map request paths to internal resources."""
    root, url_base = handler_setup
    handler = handler_cls(base_path=root, url_base=url_base)
    assert handler.path_replace == expected


@pytest.mark.parametrize("handler_cls, request_path, expected_tags", [
    (
        Test262WindowTestHandler,
        "/test262/basic.test262-test.html",
        [
            '<script src="/third_party/test262/harness/assert.js"></script>',
            '<script src="/third_party/test262/harness/sta.js"></script>',
        ]
    ),
    (
        Test262WindowTestHandler,
        "/test262/negative.test262-test.html",
        [
            "<script>test262Negative('TypeError', 'runtime')</script>",
        ]
    ),
    (
        Test262WindowTestHandler,
        "/test262/async.test262-test.html",
        [
            "<script>test262IsAsync(true)</script>",
            '<script src="/third_party/test262/harness/doneprintHandle.js"></script>',
        ]
    ),
    (
        Test262StrictWindowTestHandler,
        "/test262/teststrict.test262-test.strict.html",
        [
            '<script src="/third_party/test262/harness/propertyHelper.js"></script>',
        ]
    ),
])
def test_get_meta_and_script(handler_setup, handler_cls, request_path, expected_tags):
    """Verifies script and meta injection logic for Test262 handlers."""
    root, url_base = handler_setup
    handler = handler_cls(root, url_base)
    mock_request = _create_mock_request(request_path)
    # Combine output of _get_meta and _get_script
    output = list(handler._get_meta(mock_request)) + list(handler._get_script(mock_request))
    for item in expected_tags:
        assert item in output
    assert len(expected_tags) == len(output)


@pytest.mark.parametrize("handler_cls, request_path, expected_substrings", [
    # Test262WindowHandler: Should contain the iframe pointing to the test
    (
        Test262WindowHandler,
        "/test262/basic.test262.html",
        [
            '<script src="/resources/testharness.js"></script>',
            '<script src="/resources/testharnessreport.js"></script>',
            'window.test262HarnessDone = t.step_func_done(function(status, message)',
            '<iframe id="test262-iframe" src="/test262/basic.test262-test.html"></iframe>'
        ]
    ),
    # Test262WindowTestHandler: Should contain script tags
    (
        Test262WindowTestHandler,
        "/test262/basic.test262-test.html",
        [
            '<script src="/test262/basic.js" onerror="test262ScriptError()"></script>',
            '<script>test262Setup()</script>',
        ]
    ),
    # Test262WindowModuleTestHandler: Should contain module import and new catch logic
    (
        Test262WindowModuleTestHandler,
        "/test262/module.test262-module-test.html",
        [
            '<script type="module">',
            'test262Setup();',
            'import("/test262/module.js")',
            'setTimeout(() => { throw error; });',
        ]
    ),
    # Verification of the 'negative' replacement in the HTML
    (
        Test262WindowTestHandler,
        "/test262/negative.test262-test.html",
        [
            "<script>test262Negative('TypeError', 'runtime')</script>",
        ]
    ),
    # Verification of the 'async' replacement in the HTML
    (
        Test262WindowTestHandler,
        "/test262/async.test262-test.html",
        [
            "<script>test262IsAsync(true)</script>",
            '<script src="/third_party/test262/harness/doneprintHandle.js">'
        ]
    ),
    # Strict HTML Case: points to the .strict.js variant
    (
        Test262StrictWindowTestHandler,
        "/test262/teststrict.test262-test.strict.html",
        ['src="/test262/teststrict.test262.strict.js"']
    ),
    # Strict JS Case: The handler that serves the actual script
    (
        Test262StrictHandler,
        "/test262/teststrict.test262.strict.js",
        ['"use strict";', "console.log('hello');"]
    ),
])
def test_wrapper_content(handler_setup, handler_cls, request_path, expected_substrings):
    """Verifies generated HTML/JS content for any handler."""
    root, url_base = handler_setup
    handler = handler_cls(base_path=root, url_base=url_base)
    mock_request = _create_mock_request(request_path)
    mock_response = MagicMock()
    handler.handle_request(mock_request, mock_response)
    content = mock_response.content
    for item in expected_substrings:
        assert item in content
