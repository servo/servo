import pytest

from datetime import datetime, timedelta
from six import text_type

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
    assert isinstance(cookie["domain"], text_type)
    assert "name" in cookie
    assert isinstance(cookie["name"], text_type)
    assert "value" in cookie
    assert isinstance(cookie["value"], text_type)

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
    assert isinstance(cookie["name"], text_type)
    assert "value" in cookie
    assert isinstance(cookie["value"], text_type)
    assert "domain" in cookie
    assert isinstance(cookie["domain"], text_type)

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
    assert isinstance(cookie["name"], text_type)
    assert "value" in cookie
    assert isinstance(cookie["value"], text_type)
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
    assert isinstance(cookie["name"], text_type)
    assert "value" in cookie
    assert isinstance(cookie["value"], text_type)
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
    assert isinstance(cookie["name"], text_type)
    assert "value" in cookie
    assert isinstance(cookie["value"], text_type)
    assert "domain" in cookie
    assert isinstance(cookie["domain"], text_type)

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
    assert isinstance(cookie["name"], text_type)
    assert "value" in cookie
    assert isinstance(cookie["value"], text_type)
    assert "sameSite" in cookie
    assert isinstance(cookie["sameSite"], text_type)

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
