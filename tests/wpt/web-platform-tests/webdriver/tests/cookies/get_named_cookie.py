from tests.support.inline import inline
from tests.support.fixtures import clear_all_cookies

def test_get_named_cookie(session, url):
    session.url = url("/common/blank.html")
    session.execute_script("document.cookie = 'foo=bar'")

    result = session.transport.send("GET", "session/%s/cookie/foo" % session.session_id)
    assert result.status == 200
    assert isinstance(result.body["value"], dict)

    # table for cookie conversion
    # https://w3c.github.io/webdriver/webdriver-spec.html#dfn-table-for-cookie-conversion
    cookie = result.body["value"]
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
    assert "expiry" in cookie
    assert isinstance(cookie["expiry"], (int, long))

    assert cookie["name"] == "foo"
    assert cookie["value"] == "bar"

def test_duplicated_cookie(session, url):
    session.url = url("/common/blank.html")
    clear_all_cookies(session)
    create_cookie_request = {
        "cookie": {
            "name": "hello",
            "value": "world",
            "domain": "web-platform.test",
            "path": "/",
            "httpOnly": False,
            "secure": False
        }
    }
    result = session.transport.send("POST", "session/%s/cookie" % session.session_id, create_cookie_request)
    assert result.status == 200
    assert "value" in result.body
    assert isinstance(result.body["value"], dict)

    session.url = inline("<script>document.cookie = 'hello=newworld; domain=web-platform.test; path=/';</script>")
    result = session.transport.send("GET", "session/%s/cookie" % session.session_id)
    assert result.status == 200
    assert "value" in result.body
    assert isinstance(result.body["value"], list)
    assert len(result.body["value"]) == 1
    assert isinstance(result.body["value"][0], dict)

    cookie = result.body["value"][0]
    assert "name" in cookie
    assert isinstance(cookie["name"], basestring)
    assert "value" in cookie
    assert isinstance(cookie["value"], basestring)

    assert cookie["name"] == "hello"
    assert cookie["value"] == "newworld"

