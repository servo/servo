from datetime import datetime, timedelta

from tests.support.asserts import assert_success
from tests.support.fixtures import clear_all_cookies


def add_cookie(session, cookie):
    return session.transport.send(
        "POST", "session/{session_id}/cookie".format(**vars(session)),
        {"cookie": cookie})


def test_add_domain_cookie(session, url, server_config):
    new_cookie = {
        "name": "hello",
        "value": "world",
        "domain": server_config["browser_host"],
        "path": "/",
        "httpOnly": False,
        "secure": False
    }

    session.url = url("/common/blank.html")
    clear_all_cookies(session)

    result = add_cookie(session, new_cookie)
    assert_success(result)

    cookie = session.cookies("hello")
    assert "domain" in cookie
    assert isinstance(cookie["domain"], basestring)
    assert "name" in cookie
    assert isinstance(cookie["name"], basestring)
    assert "value" in cookie
    assert isinstance(cookie["value"], basestring)

    assert cookie["name"] == "hello"
    assert cookie["value"] == "world"
    assert cookie["domain"] == server_config["browser_host"] or \
        cookie["domain"] == ".%s" % server_config["browser_host"]


def test_add_cookie_for_ip(session, url, server_config, configuration):
    new_cookie = {
        "name": "hello",
        "value": "world",
        "domain": "127.0.0.1",
        "path": "/",
        "httpOnly": False,
        "secure": False
    }

    session.url = "http://127.0.0.1:%s/common/blank.html" % (server_config["ports"]["http"][0])
    clear_all_cookies(session)

    result = add_cookie(session, new_cookie)
    assert_success(result)

    cookie = session.cookies("hello")
    assert "name" in cookie
    assert isinstance(cookie["name"], basestring)
    assert "value" in cookie
    assert isinstance(cookie["value"], basestring)
    assert "domain" in cookie
    assert isinstance(cookie["domain"], basestring)

    assert cookie["name"] == "hello"
    assert cookie["value"] == "world"
    assert cookie["domain"] == "127.0.0.1"


def test_add_non_session_cookie(session, url):
    a_year_from_now = int(
        (datetime.utcnow() + timedelta(days=365) - datetime.utcfromtimestamp(0)).total_seconds())

    new_cookie = {
        "name": "hello",
        "value": "world",
        "expiry": a_year_from_now
    }

    session.url = url("/common/blank.html")
    clear_all_cookies(session)

    result = add_cookie(session, new_cookie)
    assert_success(result)

    cookie = session.cookies("hello")
    assert "name" in cookie
    assert isinstance(cookie["name"], basestring)
    assert "value" in cookie
    assert isinstance(cookie["value"], basestring)
    assert "expiry" in cookie
    assert isinstance(cookie["expiry"], int)

    assert cookie["name"] == "hello"
    assert cookie["value"] == "world"
    assert cookie["expiry"] == a_year_from_now


def test_add_session_cookie(session, url):
    new_cookie = {
        "name": "hello",
        "value": "world"
    }

    session.url = url("/common/blank.html")
    clear_all_cookies(session)

    result = add_cookie(session, new_cookie)
    assert_success(result)

    cookie = session.cookies("hello")
    assert "name" in cookie
    assert isinstance(cookie["name"], basestring)
    assert "value" in cookie
    assert isinstance(cookie["value"], basestring)
    if "expiry" in cookie:
        assert cookie.get("expiry") is None

    assert cookie["name"] == "hello"
    assert cookie["value"] == "world"


def test_add_session_cookie_with_leading_dot_character_in_domain(session, url, server_config):
    new_cookie = {
        "name": "hello",
        "value": "world",
        "domain": ".%s" % server_config["browser_host"]
    }

    session.url = url("/common/blank.html")
    clear_all_cookies(session)

    result = add_cookie(session, new_cookie)
    assert_success(result)

    cookie = session.cookies("hello")
    assert "name" in cookie
    assert isinstance(cookie["name"], basestring)
    assert "value" in cookie
    assert isinstance(cookie["value"], basestring)
    assert "domain" in cookie
    assert isinstance(cookie["domain"], basestring)

    assert cookie["name"] == "hello"
    assert cookie["value"] == "world"
    assert cookie["domain"] == server_config["browser_host"] or \
        cookie["domain"] == ".%s" % server_config["browser_host"]
