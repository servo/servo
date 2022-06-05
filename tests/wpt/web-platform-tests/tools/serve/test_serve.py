# mypy: allow-untyped-defs

import logging
import os
import pickle
import platform

import pytest

import localpaths  # type: ignore
from . import serve
from .serve import ConfigBuilder, inject_script


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
        assert set(lines) == {"",
                              "192.168.42.42\tfoo.bar",
                              "192.168.42.42\tfoo2.bar",
                              "192.168.42.42\ta.foo.bar",
                              "192.168.42.42\ta.foo2.bar",
                              "192.168.42.42\tb.foo.bar",
                              "192.168.42.42\tb.foo2.bar"}
        assert lines[-1] == ""

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
        assert set(lines) == {"",
                              "0.0.0.0\tx.foo.bar",
                              "0.0.0.0\tx.foo2.bar",
                              "0.0.0.0\ty.foo.bar",
                              "0.0.0.0\ty.foo2.bar",
                              "192.168.42.42\tfoo.bar",
                              "192.168.42.42\tfoo2.bar",
                              "192.168.42.42\ta.foo.bar",
                              "192.168.42.42\ta.foo2.bar",
                              "192.168.42.42\tb.foo.bar",
                              "192.168.42.42\tb.foo2.bar"}
        assert lines[-1] == ""


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
