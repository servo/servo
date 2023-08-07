import pytest

from datetime import datetime, timedelta

from webdriver.transport import Response

from tests.support.asserts import assert_error, assert_success
from tests.support.helpers import clear_all_cookies


def add_cookie(session, cookie):
    return session.transport.send(
        "POST", "session/{session_id}/cookie".format(**vars(session)),
        {"cookie": cookie})


def test_null_parameter_value(session, http):
    path = "/session/{session_id}/cookie".format(**vars(session))
    with http.post(path, None) as response:
        assert_error(Response.from_http(response), "invalid argument")


def test_null_response_value(session, url):
    new_cookie = {
        "name": "hello",
        "value": "world",
    }

    session.url = url("/common/blank.html")
    clear_all_cookies(session)

    response = add_cookie(session, new_cookie)
    value = assert_success(response)
    assert value is None


def test_no_top_browsing_context(session, closed_window):
    new_cookie = {
        "name": "hello",
        "value": "world",
    }

    response = add_cookie(session, new_cookie)
    assert_error(response, "no such window")


def test_no_browsing_context(session, closed_frame):
    new_cookie = {
        "name": "hello",
        "value": "world",
    }

    response = add_cookie(session, new_cookie)
    assert_error(response, "no such window")


@pytest.mark.parametrize(
    "page",
    [
        "about:blank",
        "blob:foo/bar",
        "data:text/html;charset=utf-8,<p>foo</p>",
        "file:///foo/bar",
        "ftp://example.org",
        "javascript:foo",
        "ws://example.org",
        "wss://example.org",
    ],
    ids=[
        "about",
        "blob",
        "data",
        "file",
        "ftp",
        "javascript",
        "websocket",
        "secure websocket",
    ],
)
def test_cookie_unsupported_scheme(session, page):
    new_cookie = {
        "name": "hello",
        "value": "world",
        "domain": page,
        "path": "/",
        "httpOnly": False,
        "secure": False
    }

    result = add_cookie(session, new_cookie)
    assert_error(result, "invalid cookie domain")


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
    assert isinstance(cookie["domain"], str)
    assert "name" in cookie
    assert isinstance(cookie["name"], str)
    assert "value" in cookie
    assert isinstance(cookie["value"], str)

    assert cookie["name"] == "hello"
    assert cookie["value"] == "world"
    assert cookie["domain"] == server_config["browser_host"] or \
        cookie["domain"] == ".%s" % server_config["browser_host"]


def test_add_cookie_for_ip(session, server_config):
    new_cookie = {
        "name": "hello",
        "value": "world",
        "domain": "127.0.0.1",
        "path": "/",
        "httpOnly": False,
        "secure": False
    }

    port = server_config["ports"]["http"][0]
    session.url = f"http://127.0.0.1:{port}/common/blank.html"

    clear_all_cookies(session)

    result = add_cookie(session, new_cookie)
    assert_success(result)

    cookie = session.cookies("hello")
    assert "name" in cookie
    assert isinstance(cookie["name"], str)
    assert "value" in cookie
    assert isinstance(cookie["value"], str)
    assert "domain" in cookie
    assert isinstance(cookie["domain"], str)

    assert cookie["name"] == "hello"
    assert cookie["value"] == "world"
    assert cookie["domain"] == "127.0.0.1"


def test_add_non_session_cookie(session, url):
    a_day_from_now = int(
        (datetime.utcnow() + timedelta(days=1) - datetime.utcfromtimestamp(0)).total_seconds())

    new_cookie = {
        "name": "hello",
        "value": "world",
        "expiry": a_day_from_now
    }

    session.url = url("/common/blank.html")
    clear_all_cookies(session)

    result = add_cookie(session, new_cookie)
    assert_success(result)

    cookie = session.cookies("hello")
    assert "name" in cookie
    assert isinstance(cookie["name"], str)
    assert "value" in cookie
    assert isinstance(cookie["value"], str)
    assert "expiry" in cookie
    assert isinstance(cookie["expiry"], int)

    assert cookie["name"] == "hello"
    assert cookie["value"] == "world"
    assert cookie["expiry"] == a_day_from_now


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
    assert isinstance(cookie["name"], str)
    assert "value" in cookie
    assert isinstance(cookie["value"], str)
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
    assert isinstance(cookie["name"], str)
    assert "value" in cookie
    assert isinstance(cookie["value"], str)
    assert "domain" in cookie
    assert isinstance(cookie["domain"], str)

    assert cookie["name"] == "hello"
    assert cookie["value"] == "world"
    assert cookie["domain"] == server_config["browser_host"] or \
        cookie["domain"] == ".%s" % server_config["browser_host"]


@pytest.mark.parametrize("same_site", ["None", "Lax", "Strict"])
def test_add_cookie_with_valid_samesite_flag(session, url, same_site):
    new_cookie = {
        "name": "hello",
        "value": "world",
        "sameSite": same_site
    }

    session.url = url("/common/blank.html")
    clear_all_cookies(session)

    result = add_cookie(session, new_cookie)
    assert_success(result)

    cookie = session.cookies("hello")
    assert "name" in cookie
    assert isinstance(cookie["name"], str)
    assert "value" in cookie
    assert isinstance(cookie["value"], str)
    assert "sameSite" in cookie
    assert isinstance(cookie["sameSite"], str)

    assert cookie["name"] == "hello"
    assert cookie["value"] == "world"
    assert cookie["sameSite"] == same_site


def test_add_cookie_with_invalid_samesite_flag(session, url):
    new_cookie = {
        "name": "hello",
        "value": "world",
        "sameSite": "invalid"
    }

    session.url = url("/common/blank.html")
    clear_all_cookies(session)

    response = add_cookie(session, new_cookie)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("same_site", [False, 12, dict()])
def test_add_cookie_with_invalid_samesite_type(session, url, same_site):
    new_cookie = {
        "name": "hello",
        "value": "world",
        "sameSite": same_site
    }

    session.url = url("/common/blank.html")
    clear_all_cookies(session)

    response = add_cookie(session, new_cookie)
    assert_error(response, "invalid argument")
