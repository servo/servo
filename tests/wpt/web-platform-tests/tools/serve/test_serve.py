import json
import os
import pickle
import platform

import pytest

import localpaths
from . import serve
from .serve import ConfigBuilder


@pytest.mark.skipif(platform.uname()[0] == "Windows",
                    reason="Expected contents are platform-dependent")
def test_make_hosts_file_nix():
    with ConfigBuilder(ports={"http": [8000]},
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
    with ConfigBuilder(ports={"http": [8000]},
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
    with ConfigBuilder() as c:
        assert c.ws_doc_root == os.path.join(localpaths.repo_root, "websockets", "handlers")


def test_init_ws_doc_root():
    with ConfigBuilder(ws_doc_root="/") as c:
        assert c.doc_root == localpaths.repo_root  # check this hasn't changed
        assert c.ws_doc_root == "/"


def test_set_ws_doc_root():
    cb = ConfigBuilder()
    cb.ws_doc_root = "/"
    with cb as c:
        assert c.doc_root == localpaths.repo_root  # check this hasn't changed
        assert c.ws_doc_root == "/"


def test_pickle():
    # Ensure that the config object can be pickled
    with ConfigBuilder() as c:
        pickle.dumps(c)


def test_config_json_length():
    # we serialize the config as JSON for pytestrunner and put it in an env
    # variable, which on Windows must have a length <= 0x7FFF (int16)
    with ConfigBuilder() as c:
        data = json.dumps(c.as_dict())
    assert len(data) <= 0x7FFF

def test_alternate_host_unspecified():
    ConfigBuilder(browser_host="web-platform.test")

@pytest.mark.parametrize("primary, alternate", [
    ("web-platform.test", "web-platform.test"),
    ("a.web-platform.test", "web-platform.test"),
    ("web-platform.test", "a.web-platform.test"),
    ("a.web-platform.test", "a.web-platform.test"),
])
def test_alternate_host_invalid(primary, alternate):
    with pytest.raises(ValueError):
        ConfigBuilder(browser_host=primary, alternate_hosts={"alt": alternate})

@pytest.mark.parametrize("primary, alternate", [
    ("web-platform.test", "not-web-platform.test"),
    ("a.web-platform.test", "b.web-platform.test"),
    ("web-platform-tests.dev", "web-platform-tests.live"),
])
def test_alternate_host_valid(primary, alternate):
    ConfigBuilder(browser_host=primary, alternate_hosts={"alt": alternate})
