import json
import logging
import pickle

from distutils.spawn import find_executable
from logging import handlers

import pytest

config = pytest.importorskip("wptserve.config")

logger = logging.getLogger()

def test_renamed_are_renamed():
    assert len(set(config._renamed_props.keys()) & set(config.ConfigBuilder._default.keys())) == 0


def test_renamed_exist():
    assert set(config._renamed_props.values()).issubset(set(config.ConfigBuilder._default.keys()))


@pytest.mark.parametrize("base, override, expected", [
    ({"a": 1}, {"a": 2}, {"a": 2}),
    ({"a": 1}, {"b": 2}, {"a": 1}),
    ({"a": {"b": 1}}, {"a": {}}, {"a": {"b": 1}}),
    ({"a": {"b": 1}}, {"a": {"b": 2}}, {"a": {"b": 2}}),
    ({"a": {"b": 1}}, {"a": {"b": 2, "c": 3}}, {"a": {"b": 2}}),
    pytest.param({"a": {"b": 1}}, {"a": 2}, {"a": 1}, marks=pytest.mark.xfail),
    pytest.param({"a": 1}, {"a": {"b": 2}}, {"a": 1}, marks=pytest.mark.xfail),
])
def test_merge_dict(base, override, expected):
    assert expected == config._merge_dict(base, override)



def test_as_dict():
    with config.ConfigBuilder(logger) as c:
        assert c.as_dict() is not None


def test_as_dict_is_json():
    with config.ConfigBuilder(logger) as c:
        assert json.dumps(c.as_dict()) is not None


def test_init_basic_prop():
    with config.ConfigBuilder(logger, browser_host="foo.bar") as c:
        assert c.browser_host == "foo.bar"


def test_init_prefixed_prop():
    with config.ConfigBuilder(logger, doc_root="/") as c:
        assert c.doc_root == "/"


def test_init_renamed_host():
    logger = logging.getLogger("test_init_renamed_host")
    logger.setLevel(logging.DEBUG)
    handler = handlers.BufferingHandler(100)
    logger.addHandler(handler)

    with config.ConfigBuilder(logger, host="foo.bar") as c:
        assert len(handler.buffer) == 1
        assert "browser_host" in handler.buffer[0].getMessage()  # check we give the new name in the message
        assert not hasattr(c, "host")
        assert c.browser_host == "foo.bar"


def test_init_bogus():
    with pytest.raises(TypeError) as e:
        config.ConfigBuilder(logger, foo=1, bar=2)
    message = e.value.args[0]
    assert "foo" in message
    assert "bar" in message


def test_getitem():
    with config.ConfigBuilder(logger, browser_host="foo.bar") as c:
        assert c["browser_host"] == "foo.bar"


def test_no_setitem():
    with config.ConfigBuilder(logger) as c:
        with pytest.raises(ValueError):
            c["browser_host"] = "foo.bar"


def test_iter():
    with config.ConfigBuilder(logger) as c:
        s = set(iter(c))
        assert "browser_host" in s
        assert "host" not in s
        assert "__getitem__" not in s
        assert "_browser_host" not in s


def test_assignment():
    cb = config.ConfigBuilder(logger)
    cb.browser_host = "foo.bar"
    with cb as c:
        assert c.browser_host == "foo.bar"


def test_update_basic():
    cb = config.ConfigBuilder(logger)
    cb.update({"browser_host": "foo.bar"})
    with cb as c:
        assert c.browser_host == "foo.bar"


def test_update_prefixed():
    cb = config.ConfigBuilder(logger)
    cb.update({"doc_root": "/"})
    with cb as c:
        assert c.doc_root == "/"


def test_update_renamed_host():
    logger = logging.getLogger("test_update_renamed_host")
    logger.setLevel(logging.DEBUG)
    handler = handlers.BufferingHandler(100)
    logger.addHandler(handler)

    cb = config.ConfigBuilder(logger)
    assert len(handler.buffer) == 0

    cb.update({"host": "foo.bar"})

    with cb as c:
        assert len(handler.buffer) == 1
        assert "browser_host" in handler.buffer[0].getMessage()  # check we give the new name in the message
        assert not hasattr(c, "host")
        assert c.browser_host == "foo.bar"


def test_update_bogus():
    cb = config.ConfigBuilder(logger)
    with pytest.raises(KeyError):
        cb.update({"foobar": 1})


def test_ports_auto():
    with config.ConfigBuilder(logger,
                              ports={"http": ["auto"]},
                              ssl={"type": "none"}) as c:
        ports = c.ports
        assert set(ports.keys()) == {"http"}
        assert len(ports["http"]) == 1
        assert isinstance(ports["http"][0], int)


def test_ports_auto_mutate():
    cb = config.ConfigBuilder(logger,
                              ports={"http": [1001]},
                              ssl={"type": "none"})
    cb.ports = {"http": ["auto"]}
    with cb as c:
        new_ports = c.ports
        assert set(new_ports.keys()) == {"http"}
        assert len(new_ports["http"]) == 1
        assert isinstance(new_ports["http"][0], int)


def test_ports_explicit():
    with config.ConfigBuilder(logger,
                              ports={"http": [1001]},
                              ssl={"type": "none"}) as c:
        ports = c.ports
        assert set(ports.keys()) == {"http"}
        assert ports["http"] == [1001]


def test_ports_no_ssl():
    with config.ConfigBuilder(logger,
                              ports={"http": [1001], "https": [1002], "ws": [1003], "wss": [1004]},
                              ssl={"type": "none"}) as c:
        ports = c.ports
        assert set(ports.keys()) == {"http", "ws"}
        assert ports["http"] == [1001]
        assert ports["ws"] == [1003]


@pytest.mark.skipif(find_executable("openssl") is None,
                    reason="requires OpenSSL")
def test_ports_openssl():
    with config.ConfigBuilder(logger,
                              ports={"http": [1001], "https": [1002], "ws": [1003], "wss": [1004]},
                              ssl={"type": "openssl"}) as c:
        ports = c.ports
        assert set(ports.keys()) == {"http", "https", "ws", "wss"}
        assert ports["http"] == [1001]
        assert ports["https"] == [1002]
        assert ports["ws"] == [1003]
        assert ports["wss"] == [1004]


def test_init_doc_root():
    with config.ConfigBuilder(logger, doc_root="/") as c:
        assert c.doc_root == "/"


def test_set_doc_root():
    cb = config.ConfigBuilder(logger)
    cb.doc_root = "/"
    with cb as c:
        assert c.doc_root == "/"


def test_server_host_from_browser_host():
    with config.ConfigBuilder(logger, browser_host="foo.bar") as c:
        assert c.server_host == "foo.bar"


def test_init_server_host():
    with config.ConfigBuilder(logger, server_host="foo.bar") as c:
        assert c.browser_host == "localhost"  # check this hasn't changed
        assert c.server_host == "foo.bar"


def test_set_server_host():
    cb = config.ConfigBuilder(logger)
    cb.server_host = "/"
    with cb as c:
        assert c.browser_host == "localhost"  # check this hasn't changed
        assert c.server_host == "/"


def test_domains():
    with config.ConfigBuilder(logger,
                              browser_host="foo.bar",
                              alternate_hosts={"alt": "foo2.bar"},
                              subdomains={"a", "b"},
                              not_subdomains={"x", "y"}) as c:
        assert c.domains == {
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


def test_not_domains():
    with config.ConfigBuilder(logger,
                              browser_host="foo.bar",
                              alternate_hosts={"alt": "foo2.bar"},
                              subdomains={"a", "b"},
                              not_subdomains={"x", "y"}) as c:
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


def test_domains_not_domains_intersection():
    with config.ConfigBuilder(logger,
                              browser_host="foo.bar",
                              alternate_hosts={"alt": "foo2.bar"},
                              subdomains={"a", "b"},
                              not_subdomains={"x", "y"}) as c:
        domains = c.domains
        not_domains = c.not_domains
        assert len(set(domains.keys()) ^ set(not_domains.keys())) == 0
        for host in domains.keys():
            host_domains = domains[host]
            host_not_domains = not_domains[host]
            assert len(set(host_domains.keys()) & set(host_not_domains.keys())) == 0
            assert len(set(host_domains.values()) & set(host_not_domains.values())) == 0


def test_all_domains():
    with config.ConfigBuilder(logger,
                              browser_host="foo.bar",
                              alternate_hosts={"alt": "foo2.bar"},
                              subdomains={"a", "b"},
                              not_subdomains={"x", "y"}) as c:
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


def test_domains_set():
    with config.ConfigBuilder(logger,
                              browser_host="foo.bar",
                              alternate_hosts={"alt": "foo2.bar"},
                              subdomains={"a", "b"},
                              not_subdomains={"x", "y"}) as c:
        domains_set = c.domains_set
        assert domains_set == {
            "foo.bar",
            "a.foo.bar",
            "b.foo.bar",
            "foo2.bar",
            "a.foo2.bar",
            "b.foo2.bar",
        }


def test_not_domains_set():
    with config.ConfigBuilder(logger,
                              browser_host="foo.bar",
                              alternate_hosts={"alt": "foo2.bar"},
                              subdomains={"a", "b"},
                              not_subdomains={"x", "y"}) as c:
        not_domains_set = c.not_domains_set
        assert not_domains_set == {
            "x.foo.bar",
            "y.foo.bar",
            "x.foo2.bar",
            "y.foo2.bar",
        }


def test_all_domains_set():
    with config.ConfigBuilder(logger,
                              browser_host="foo.bar",
                              alternate_hosts={"alt": "foo2.bar"},
                              subdomains={"a", "b"},
                              not_subdomains={"x", "y"}) as c:
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


def test_ssl_env_none():
    with config.ConfigBuilder(logger, ssl={"type": "none"}) as c:
        assert c.ssl_config is None


def test_ssl_env_openssl():
    # TODO: this currently actually tries to start OpenSSL, which isn't ideal
    # with config.ConfigBuilder(ssl={"type": "openssl", "openssl": {"openssl_binary": "foobar"}}) as c:
    #     assert c.ssl_env is not None
    #     assert c.ssl_env.ssl_enabled is True
    #     assert c.ssl_env.binary == "foobar"
    pass


def test_ssl_env_bogus():
    with pytest.raises(ValueError):
        with config.ConfigBuilder(logger, ssl={"type": "foobar"}):
            pass


def test_pickle():
    # Ensure that the config object can be pickled
    with config.ConfigBuilder(logger) as c:
        pickle.dumps(c)
