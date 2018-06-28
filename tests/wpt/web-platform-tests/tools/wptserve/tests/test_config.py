import logging
import pickle
import sys
from logging import handlers

import pytest

config = pytest.importorskip("wptserve.config")


@pytest.mark.xfail(sys.version_info >= (3,), reason="wptserve only works on Py2")
def test_renamed_are_renamed():
    assert len(set(config._renamed_props.viewkeys()) & set(config.Config._default.viewkeys())) == 0


@pytest.mark.xfail(sys.version_info >= (3,), reason="wptserve only works on Py2")
def test_renamed_exist():
    assert set(config._renamed_props.viewvalues()).issubset(set(config.Config._default.viewkeys()))


@pytest.mark.parametrize("base, override, expected", [
    ({"a": 1}, {"a": 2}, {"a": 2}),
    ({"a": 1}, {"b": 2}, {"a": 1}),
    ({"a": {"b": 1}}, {"a": {}}, {"a": {"b": 1}}),
    ({"a": {"b": 1}}, {"a": {"b": 2}}, {"a": {"b": 2}}),
    ({"a": {"b": 1}}, {"a": {"b": 2, "c": 3}}, {"a": {"b": 2}}),
    pytest.param({"a": {"b": 1}}, {"a": 2}, {"a": 1}, marks=pytest.mark.xfail),
    pytest.param({"a": 1}, {"a": {"b": 2}}, {"a": 1}, marks=pytest.mark.xfail),
])
@pytest.mark.xfail(sys.version_info >= (3,), reason="wptserve only works on Py2")
def test_merge_dict(base, override, expected):
    assert expected == config._merge_dict(base, override)


@pytest.mark.xfail(sys.version_info >= (3,), reason="wptserve only works on Py2")
def test_logger_created():
    c = config.Config()
    assert c.logger is not None


@pytest.mark.xfail(sys.version_info >= (3,), reason="wptserve only works on Py2")
def test_logger_preserved():
    logger = logging.getLogger("test_logger_preserved")
    logger.setLevel(logging.DEBUG)

    c = config.Config(logger=logger)
    assert c.logger is logger


@pytest.mark.xfail(sys.version_info >= (3,), reason="wptserve only works on Py2")
def test_init_basic_prop():
    c = config.Config(browser_host="foo.bar")
    assert c.browser_host == "foo.bar"


@pytest.mark.xfail(sys.version_info >= (3,), reason="wptserve only works on Py2")
def test_init_prefixed_prop():
    c = config.Config(doc_root="/")
    assert c._doc_root == "/"


@pytest.mark.xfail(sys.version_info >= (3,), reason="wptserve only works on Py2")
def test_init_renamed_host():
    logger = logging.getLogger("test_init_renamed_host")
    logger.setLevel(logging.DEBUG)
    handler = handlers.BufferingHandler(100)
    logger.addHandler(handler)

    c = config.Config(logger=logger, host="foo.bar")
    assert c.logger is logger
    assert len(handler.buffer) == 1
    assert "browser_host" in handler.buffer[0].getMessage()  # check we give the new name in the message
    assert not hasattr(c, "host")
    assert c.browser_host == "foo.bar"


@pytest.mark.xfail(sys.version_info >= (3,), reason="wptserve only works on Py2")
def test_init_bogus():
    with pytest.raises(TypeError) as e:
        config.Config(foo=1, bar=2)
    assert "foo" in e.value.message
    assert "bar" in e.value.message


@pytest.mark.xfail(sys.version_info >= (3,), reason="wptserve only works on Py2")
def test_getitem():
    c = config.Config(browser_host="foo.bar")
    assert c["browser_host"] == "foo.bar"


@pytest.mark.xfail(sys.version_info >= (3,), reason="wptserve only works on Py2")
def test_no_setitem():
    c = config.Config()
    with pytest.raises(TypeError):
        c["browser_host"] = "foo.bar"


@pytest.mark.xfail(sys.version_info >= (3,), reason="wptserve only works on Py2")
def test_iter():
    c = config.Config()
    s = set(iter(c))
    assert "browser_host" in s
    assert "host" not in s
    assert "__getitem__" not in s
    assert "_browser_host" not in s


@pytest.mark.xfail(sys.version_info >= (3,), reason="wptserve only works on Py2")
def test_assignment():
    c = config.Config()
    c.browser_host = "foo.bar"
    assert c.browser_host == "foo.bar"


@pytest.mark.xfail(sys.version_info >= (3,), reason="wptserve only works on Py2")
def test_update_basic():
    c = config.Config()
    c.update({"browser_host": "foo.bar"})
    assert c.browser_host == "foo.bar"


@pytest.mark.xfail(sys.version_info >= (3,), reason="wptserve only works on Py2")
def test_update_prefixed():
    c = config.Config()
    c.update({"doc_root": "/"})
    assert c._doc_root == "/"


@pytest.mark.xfail(sys.version_info >= (3,), reason="wptserve only works on Py2")
def test_update_renamed_host():
    logger = logging.getLogger("test_update_renamed_host")
    logger.setLevel(logging.DEBUG)
    handler = handlers.BufferingHandler(100)
    logger.addHandler(handler)

    c = config.Config(logger=logger)
    assert c.logger is logger
    assert len(handler.buffer) == 0

    c.update({"host": "foo.bar"})

    assert len(handler.buffer) == 1
    assert "browser_host" in handler.buffer[0].getMessage()  # check we give the new name in the message
    assert not hasattr(c, "host")
    assert c.browser_host == "foo.bar"


@pytest.mark.xfail(sys.version_info >= (3,), reason="wptserve only works on Py2")
def test_update_bogus():
    c = config.Config()
    with pytest.raises(KeyError):
        c.update({"foobar": 1})


@pytest.mark.xfail(sys.version_info >= (3,), reason="wptserve only works on Py2")
def test_ports_auto():
    c = config.Config(ports={"http": ["auto"]},
                      ssl={"type": "none"})
    ports = c.ports
    assert set(ports.keys()) == {"http"}
    assert len(ports["http"]) == 1
    assert isinstance(ports["http"][0], int)


@pytest.mark.xfail(sys.version_info >= (3,), reason="wptserve only works on Py2")
def test_ports_auto_mutate():
    c = config.Config(ports={"http": [1001]},
                      ssl={"type": "none"})
    orig_ports = c.ports
    assert set(orig_ports.keys()) == {"http"}
    assert orig_ports["http"] == [1001]

    c.ports = {"http": ["auto"]}
    new_ports = c.ports
    assert set(new_ports.keys()) == {"http"}
    assert len(new_ports["http"]) == 1
    assert isinstance(new_ports["http"][0], int)


@pytest.mark.xfail(sys.version_info >= (3,), reason="wptserve only works on Py2")
def test_ports_auto_roundtrip():
    c = config.Config(ports={"http": ["auto"]},
                      ssl={"type": "none"})
    old_ports = c.ports
    c.ports = old_ports
    new_ports = c.ports
    assert old_ports == new_ports


@pytest.mark.xfail(sys.version_info >= (3,), reason="wptserve only works on Py2")
def test_ports_idempotent():
    c = config.Config(ports={"http": ["auto"]},
                      ssl={"type": "none"})
    ports_a = c.ports
    ports_b = c.ports
    assert ports_a == ports_b


@pytest.mark.xfail(sys.version_info >= (3,), reason="wptserve only works on Py2")
def test_ports_explicit():
    c = config.Config(ports={"http": [1001]},
                      ssl={"type": "none"})
    ports = c.ports
    assert set(ports.keys()) == {"http"}
    assert ports["http"] == [1001]


@pytest.mark.xfail(sys.version_info >= (3,), reason="wptserve only works on Py2")
def test_ports_no_ssl():
    c = config.Config(ports={"http": [1001], "https": [1002], "ws": [1003], "wss": [1004]},
                      ssl={"type": "none"})
    ports = c.ports
    assert set(ports.keys()) == {"http", "https", "ws", "wss"}
    assert ports["http"] == [1001]
    assert ports["https"] == [None]
    assert ports["ws"] == [1003]
    assert ports["wss"] == [None]


@pytest.mark.xfail(sys.version_info >= (3,), reason="wptserve only works on Py2")
def test_ports_openssl():
    c = config.Config(ports={"http": [1001], "https": [1002], "ws": [1003], "wss": [1004]},
                      ssl={"type": "openssl"})
    ports = c.ports
    assert set(ports.keys()) == {"http", "https", "ws", "wss"}
    assert ports["http"] == [1001]
    assert ports["https"] == [1002]
    assert ports["ws"] == [1003]
    assert ports["wss"] == [1004]


@pytest.mark.xfail(sys.version_info >= (3,), reason="wptserve only works on Py2")
def test_init_doc_root():
    c = config.Config(doc_root="/")
    assert c._doc_root == "/"
    assert c.doc_root == "/"


@pytest.mark.xfail(sys.version_info >= (3,), reason="wptserve only works on Py2")
def test_set_doc_root():
    c = config.Config()
    c.doc_root = "/"
    assert c._doc_root == "/"
    assert c.doc_root == "/"


@pytest.mark.xfail(sys.version_info >= (3,), reason="wptserve only works on Py2")
def test_server_host_from_browser_host():
    c = config.Config(browser_host="foo.bar")
    assert c.server_host == "foo.bar"


@pytest.mark.xfail(sys.version_info >= (3,), reason="wptserve only works on Py2")
def test_init_server_host():
    c = config.Config(server_host="foo.bar")
    assert c.browser_host == "localhost"  # check this hasn't changed
    assert c._server_host == "foo.bar"
    assert c.server_host == "foo.bar"


@pytest.mark.xfail(sys.version_info >= (3,), reason="wptserve only works on Py2")
def test_set_server_host():
    c = config.Config()
    c.server_host = "/"
    assert c.browser_host == "localhost"  # check this hasn't changed
    assert c._server_host == "/"
    assert c.server_host == "/"


@pytest.mark.xfail(sys.version_info >= (3,), reason="wptserve only works on Py2")
def test_domains():
    c = config.Config(browser_host="foo.bar",
                      alternate_hosts={"alt": "foo2.bar"},
                      subdomains={"a", "b"},
                      not_subdomains={"x", "y"})
    domains = c.domains
    assert domains == {
        "": {
            "": "foo.bar",
            "a": "a.foo.bar",
            "b": "b.foo.bar",
        },
        "alt": {
            "": "foo2.bar",
            "a": "a.foo2.bar",
            "b": "b.foo2.bar",
        },
    }


@pytest.mark.xfail(sys.version_info >= (3,), reason="wptserve only works on Py2")
def test_not_domains():
    c = config.Config(browser_host="foo.bar",
                      alternate_hosts={"alt": "foo2.bar"},
                      subdomains={"a", "b"},
                      not_subdomains={"x", "y"})
    not_domains = c.not_domains
    assert not_domains == {
        "": {
            "x": "x.foo.bar",
            "y": "y.foo.bar",
        },
        "alt": {
            "x": "x.foo2.bar",
            "y": "y.foo2.bar",
        },
    }


@pytest.mark.xfail(sys.version_info >= (3,), reason="wptserve only works on Py2")
def test_domains_not_domains_intersection():
    c = config.Config(browser_host="foo.bar",
                      alternate_hosts={"alt": "foo2.bar"},
                      subdomains={"a", "b"},
                      not_subdomains={"x", "y"})
    domains = c.domains
    not_domains = c.not_domains
    assert len(set(domains.iterkeys()) ^ set(not_domains.iterkeys())) == 0
    for host in domains.iterkeys():
        host_domains = domains[host]
        host_not_domains = not_domains[host]
        assert len(set(host_domains.iterkeys()) & set(host_not_domains.iterkeys())) == 0
        assert len(set(host_domains.itervalues()) & set(host_not_domains.itervalues())) == 0


@pytest.mark.xfail(sys.version_info >= (3,), reason="wptserve only works on Py2")
def test_all_domains():
    c = config.Config(browser_host="foo.bar",
                      alternate_hosts={"alt": "foo2.bar"},
                      subdomains={"a", "b"},
                      not_subdomains={"x", "y"})
    all_domains = c.all_domains
    assert all_domains == {
        "": {
            "": "foo.bar",
            "a": "a.foo.bar",
            "b": "b.foo.bar",
            "x": "x.foo.bar",
            "y": "y.foo.bar",
        },
        "alt": {
            "": "foo2.bar",
            "a": "a.foo2.bar",
            "b": "b.foo2.bar",
            "x": "x.foo2.bar",
            "y": "y.foo2.bar",
        },
    }


@pytest.mark.xfail(sys.version_info >= (3,), reason="wptserve only works on Py2")
def test_domains_set():
    c = config.Config(browser_host="foo.bar",
                      alternate_hosts={"alt": "foo2.bar"},
                      subdomains={"a", "b"},
                      not_subdomains={"x", "y"})
    domains_set = c.domains_set
    assert domains_set == {
        "foo.bar",
        "a.foo.bar",
        "b.foo.bar",
        "foo2.bar",
        "a.foo2.bar",
        "b.foo2.bar",
    }


@pytest.mark.xfail(sys.version_info >= (3,), reason="wptserve only works on Py2")
def test_not_domains_set():
    c = config.Config(browser_host="foo.bar",
                      alternate_hosts={"alt": "foo2.bar"},
                      subdomains={"a", "b"},
                      not_subdomains={"x", "y"})
    not_domains_set = c.not_domains_set
    assert not_domains_set == {
        "x.foo.bar",
        "y.foo.bar",
        "x.foo2.bar",
        "y.foo2.bar",
    }


@pytest.mark.xfail(sys.version_info >= (3,), reason="wptserve only works on Py2")
def test_all_domains_set():
    c = config.Config(browser_host="foo.bar",
                      alternate_hosts={"alt": "foo2.bar"},
                      subdomains={"a", "b"},
                      not_subdomains={"x", "y"})
    all_domains_set = c.all_domains_set
    assert all_domains_set == {
        "foo.bar",
        "a.foo.bar",
        "b.foo.bar",
        "x.foo.bar",
        "y.foo.bar",
        "foo2.bar",
        "a.foo2.bar",
        "b.foo2.bar",
        "x.foo2.bar",
        "y.foo2.bar",
    }


@pytest.mark.xfail(sys.version_info >= (3,), reason="wptserve only works on Py2")
def test_ssl_env_override():
    c = config.Config(override_ssl_env="foobar")
    assert c.ssl_env == "foobar"


@pytest.mark.xfail(sys.version_info >= (3,), reason="wptserve only works on Py2")
def test_ssl_env_none():
    c = config.Config(ssl={"type": "none"})
    assert c.ssl_env is not None
    assert c.ssl_env.ssl_enabled is False


@pytest.mark.xfail(sys.version_info >= (3,), reason="wptserve only works on Py2")
def test_ssl_env_openssl():
    c = config.Config(ssl={"type": "openssl", "openssl": {"openssl_binary": "foobar"}})
    assert c.ssl_env is not None
    assert c.ssl_env.ssl_enabled is True
    assert c.ssl_env.binary == "foobar"


@pytest.mark.xfail(sys.version_info >= (3,), reason="wptserve only works on Py2")
def test_ssl_env_bogus():
    c = config.Config(ssl={"type": "foobar"})
    with pytest.raises(ValueError):
        c.ssl_env


@pytest.mark.xfail(sys.version_info >= (3,), reason="wptserve only works on Py2")
def test_pickle():
    # Ensure that the config object can be pickled
    pickle.dumps(config.Config())
