import pickle
import os

import pytest

import localpaths
from . import serve
from .serve import Config


def test_make_hosts_file():
    c = Config(browser_host="foo.bar", alternate_hosts={"alt": "foo2.bar"})
    hosts = serve.make_hosts_file(c, "192.168.42.42")
    lines = hosts.split("\n")
    assert set(lines) == {"",
                          "0.0.0.0\tnonexistent-origin.foo.bar",
                          "0.0.0.0\tnonexistent-origin.foo2.bar",
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
    c = Config()
    assert c.ws_doc_root == os.path.join(localpaths.repo_root, "websockets", "handlers")


def test_init_ws_doc_root():
    c = Config(ws_doc_root="/")
    assert c.doc_root == localpaths.repo_root  # check this hasn't changed
    assert c._ws_doc_root == "/"
    assert c.ws_doc_root == "/"


def test_set_ws_doc_root():
    c = Config()
    c.ws_doc_root = "/"
    assert c.doc_root == localpaths.repo_root  # check this hasn't changed
    assert c._ws_doc_root == "/"
    assert c.ws_doc_root == "/"


def test_pickle():
    # Ensure that the config object can be pickled
    pickle.dumps(Config())
