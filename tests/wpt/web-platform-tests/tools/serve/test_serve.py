import pickle
import platform
import os

import pytest

import localpaths
from . import serve
from .serve import ConfigBuilder


@pytest.mark.skipif(platform.uname()[0] == "Windows",
                    reason="Expected contents are platform-dependent")
def test_make_hosts_file_nix():
    with ConfigBuilder(ports={"http": [8000]},
                       browser_host="foo.bar",
                       alternate_hosts={"alt": "foo2.bar"}) as c:
        hosts = serve.make_hosts_file(c, "192.168.42.42")
        lines = hosts.split("\n")
        assert set(lines) == {"",
                              "192.168.42.42\tfoo.bar",
                              "192.168.42.42\tfoo2.bar",
                              "192.168.42.42\twww.foo.bar",
                              "192.168.42.42\twww.foo2.bar",
                              "192.168.42.42\twww1.foo.bar",
                              "192.168.42.42\twww1.foo2.bar",
                              "192.168.42.42\twww2.foo.bar",
                              "192.168.42.42\twww2.foo2.bar",
                              "192.168.42.42\txn--lve-6lad.foo.bar",
                              "192.168.42.42\txn--lve-6lad.foo2.bar",
                              "192.168.42.42\txn--n8j6ds53lwwkrqhv28a.foo.bar",
                              "192.168.42.42\txn--n8j6ds53lwwkrqhv28a.foo2.bar"}
        assert lines[-1] == ""

@pytest.mark.skipif(platform.uname()[0] != "Windows",
                    reason="Expected contents are platform-dependent")
def test_make_hosts_file_windows():
    with ConfigBuilder(ports={"http": [8000]},
                       browser_host="foo.bar",
                       alternate_hosts={"alt": "foo2.bar"}) as c:
        hosts = serve.make_hosts_file(c, "192.168.42.42")
        lines = hosts.split("\n")
        assert set(lines) == {"",
                              "0.0.0.0\tnonexistent.foo.bar",
                              "0.0.0.0\tnonexistent.foo2.bar",
                              "192.168.42.42\tfoo.bar",
                              "192.168.42.42\tfoo2.bar",
                              "192.168.42.42\twww.foo.bar",
                              "192.168.42.42\twww.foo2.bar",
                              "192.168.42.42\twww1.foo.bar",
                              "192.168.42.42\twww1.foo2.bar",
                              "192.168.42.42\twww2.foo.bar",
                              "192.168.42.42\twww2.foo2.bar",
                              "192.168.42.42\txn--lve-6lad.foo.bar",
                              "192.168.42.42\txn--lve-6lad.foo2.bar",
                              "192.168.42.42\txn--n8j6ds53lwwkrqhv28a.foo.bar",
                              "192.168.42.42\txn--n8j6ds53lwwkrqhv28a.foo2.bar"}
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
