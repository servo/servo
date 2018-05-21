from datetime import datetime, timedelta

from tests.support.asserts import assert_success
from tests.support.fixtures import clear_all_cookies
from tests.support.inline import inline


def get_named_cookie(session, name):
    return session.transport.send(
        "GET", "session/{session_id}/cookie/{name}".format(
            session_id=session.session_id,
            name=name))


def test_get_named_session_cookie(session, url):
    session.url = url("/common/blank.html")
    clear_all_cookies(session)
    session.execute_script("document.cookie = 'foo=bar'")

    result = get_named_cookie(session, "foo")
    cookie = assert_success(result)
    assert isinstance(cookie, dict)

    # table for cookie conversion
    # https://w3c.github.io/webdriver/webdriver-spec.html#dfn-table-for-cookie-conversion
    assert "name" in cookie
    assert isinstance(cookie["name"], basestring)
    assert "value" in cookie
    assert isinstance(cookie["value"], basestring)
    assert "path" in cookie
    assert isinstance(cookie["path"], basestring)
    assert "domain" in cookie
    assert isinstance(cookie["domain"], basestring)
    assert "secure" in cookie
    assert isinstance(cookie["secure"], bool)
    assert "httpOnly" in cookie
    assert isinstance(cookie["httpOnly"], bool)
    if "expiry" in cookie:
        assert cookie.get("expiry") is None

    assert cookie["name"] == "foo"
    assert cookie["value"] == "bar"


def test_get_named_cookie(session, url):
    session.url = url("/common/blank.html")
    clear_all_cookies(session)

    # same formatting as Date.toUTCString() in javascript
    utc_string_format = "%a, %d %b %Y %H:%M:%S"
    a_year_from_now = (datetime.utcnow() + timedelta(days=365)).strftime(utc_string_format)
    session.execute_script("document.cookie = 'foo=bar;expires=%s'" % a_year_from_now)

    result = get_named_cookie(session, "foo")
    cookie = assert_success(result)
    assert isinstance(cookie, dict)

    assert "name" in cookie
    assert isinstance(cookie["name"], basestring)
    assert "value" in cookie
    assert isinstance(cookie["value"], basestring)
    assert "expiry" in cookie
    assert isinstance(cookie["expiry"], (int, long))

    assert cookie["name"] == "foo"
    assert cookie["value"] == "bar"
    # convert from seconds since epoch
    assert datetime.utcfromtimestamp(
        cookie["expiry"]).strftime(utc_string_format) == a_year_from_now


def test_duplicated_cookie(session, url, server_config):
    new_cookie = {
        "name": "hello",
        "value": "world",
        "domain": server_config["browser_host"],
        "path": "/",
        "http_only": False,
        "secure": False
    }

    session.url = url("/common/blank.html")
    clear_all_cookies(session)

    session.set_cookie(**new_cookie)
    session.url = inline("""
      <script>
        document.cookie = '{name}=newworld; domain={domain}; path=/';
      </script>""".format(
        name=new_cookie["name"],
        domain=server_config["browser_host"]))

    result = get_named_cookie(session, new_cookie["name"])
    cookie = assert_success(result)
    assert isinstance(cookie, dict)

    assert "name" in cookie
    assert isinstance(cookie["name"], basestring)
    assert "value" in cookie
    assert isinstance(cookie["value"], basestring)

    assert cookie["name"] == new_cookie["name"]
    assert cookie["value"] == "newworld"
